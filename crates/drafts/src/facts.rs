//! The leaflet fact model — EAV rows over uuid entities
//! (`experiments/02-draft-internals/REPORT.md` §1).
//!
//! A fact row is `{id, entity, attribute, data, version, author_did,
//! created_at, updated_at[, indexes]}`; `data` is a tagged union
//! `{type, …payload}` (`src/replicache/attributes.ts:592-735`). This module
//! keeps the payload shapes the extraction spine needs plus a
//! [`FactData::Other`] tolerance for everything else — unknown `data` types
//! parse (and are ignored) rather than failing the pull.
//!
//! [`ScanIndex`] is the pull-side shape of Leaflet's `scanIndexLocal`
//! (`src/replicache/utils.ts`): `entity → attribute → [facts]`, with
//! insertion order preserved. Cardinality-`one` attributes read `[0]`
//! (`.one`); `many` reads all (`.eav`).

use std::collections::HashMap;

/// A freehand stroke payload (`drawing/stroke`, type `ink-stroke`).
#[derive(Debug, Clone, PartialEq)]
pub struct InkStroke {
    /// Flattened x, y, pressure triples; pressure runs 0 to 1000.
    pub points: Vec<f64>,
    pub color: String,
    pub size: f64,
    pub simulate_pressure: Option<bool>,
    /// Flattened x, y pairs, joined back to the first.
    pub fill: Option<StrokeFill>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeFill {
    pub points: Vec<f64>,
    pub color: String,
}

/// A `drawing/view-box` payload.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// An image payload (`block/image`, `link/preview`, …): `src` is the
/// pre-upload URL an `uploadImage` hook would resolve.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageData {
    pub src: String,
    pub height: f64,
    pub width: f64,
}

/// The `data` tagged union, restricted to the payloads the extraction spine
/// reads. Union-typed attributes (`page-type-union`, `block-type-union`,
/// `list-style-union`, …) all share the `{type, value: string}` shape and
/// land in [`FactData::Union`] with their wire tag kept for diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub enum FactData {
    /// `text` — the yjs base64 payload of `block/text`.
    Text { value: String },
    /// `string`.
    String { value: String },
    /// `string-array`.
    StringArray { value: Vec<String> },
    /// `boolean`.
    Boolean { value: bool },
    /// `number` (JSON numbers; integers and floats ride the same lane).
    Number { value: f64 },
    /// `reference` — unordered entity pointer (`block/card`).
    Reference { value: String },
    /// `ordered-reference` — entity pointer with a fractional-index
    /// `position` (`root/page`, `card/block`, `gallery/image`).
    OrderedReference { value: String, position: String },
    /// `spatial-reference` — canvas pointer with an {x, y} position.
    SpatialReference { value: String, x: f64, y: f64 },
    /// `image`.
    Image(ImageData),
    /// `bluesky-post` — the `app.bsky.feed.getPostThread` thread blob.
    BlueskyPost { value: serde_json::Value },
    /// `ink-stroke` (`drawing/stroke`).
    InkStroke { value: InkStroke },
    /// `view-box` (`drawing/view-box`).
    ViewBox { value: ViewBox },
    /// `date-time` and every `*-union` attribute: value string + wire tag.
    Union { kind: String, value: String },
    /// Any other payload (`awareness`, `color`, …) — tolerated, unused.
    Other { kind: String },
}

impl FactData {
    /// Parse the wire `data` object. Unknown `type` tags yield
    /// [`FactData::Other`]; known tags with missing payload members also
    /// degrade to `Other` rather than erroring (a fact we cannot read is
    /// skipped by extraction, not fatal to the pull).
    pub fn parse(value: &serde_json::Value) -> FactData {
        use serde_json::Value;
        let kind = match value.get("type").and_then(Value::as_str) {
            Some(kind) => kind.to_string(),
            None => return FactData::Other { kind: String::new() },
        };
        let string_at = |key: &str| value.get(key).and_then(Value::as_str);
        let number_at = |key: &str| value.get(key).and_then(Value::as_f64);
        let bool_at = |key: &str| value.get(key).and_then(Value::as_bool);
        match kind.as_str() {
            "text" | "string" => match string_at("value") {
                Some(s) if kind == "text" => FactData::Text { value: s.to_string() },
                Some(s) => FactData::String { value: s.to_string() },
                None => FactData::Other { kind },
            },
            "string-array" => match value.get("value").and_then(Value::as_array) {
                Some(items) => FactData::StringArray {
                    value: items.iter().filter_map(Value::as_str).map(str::to_string).collect(),
                },
                None => FactData::Other { kind },
            },
            "boolean" => match bool_at("value") {
                Some(b) => FactData::Boolean { value: b },
                None => FactData::Other { kind },
            },
            "number" => match number_at("value") {
                Some(n) => FactData::Number { value: n },
                None => FactData::Other { kind },
            },
            "reference" => match string_at("value") {
                Some(s) => FactData::Reference { value: s.to_string() },
                None => FactData::Other { kind },
            },
            "ordered-reference" => match (string_at("value"), string_at("position")) {
                (Some(v), Some(p)) => FactData::OrderedReference { value: v.to_string(), position: p.to_string() },
                _ => FactData::Other { kind },
            },
            "spatial-reference" => {
                let pos = value.get("position");
                match (string_at("value"), pos.and_then(|p| p.get("x")).and_then(Value::as_f64), pos.and_then(|p| p.get("y")).and_then(Value::as_f64)) {
                    (Some(v), Some(x), Some(y)) => FactData::SpatialReference { value: v.to_string(), x, y },
                    _ => FactData::Other { kind },
                }
            }
            "image" => match (string_at("src"), number_at("height"), number_at("width")) {
                (Some(src), Some(height), Some(width)) => FactData::Image(ImageData { src: src.to_string(), height, width }),
                _ => FactData::Other { kind },
            },
            "bluesky-post" => match value.get("value") {
                Some(v) => FactData::BlueskyPost { value: v.clone() },
                None => FactData::Other { kind },
            },
            "ink-stroke" => {
                let stroke = value.get("value");
                let points = stroke
                    .and_then(|s| s.get("points"))
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(Value::as_f64).collect::<Vec<_>>());
                let fill = stroke.and_then(|s| s.get("fill")).and_then(|f| {
                    let points = f.get("points").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_f64).collect::<Vec<_>>());
                    let color = f.get("color").and_then(Value::as_str);
                    match (points, color) {
                        (Some(points), Some(color)) => Some(StrokeFill { points, color: color.to_string() }),
                        _ => None,
                    }
                });
                match (points, stroke.and_then(|s| s.get("color")).and_then(Value::as_str), stroke.and_then(|s| s.get("size")).and_then(Value::as_f64)) {
                    (Some(points), Some(color), Some(size)) => FactData::InkStroke {
                        value: InkStroke {
                            points,
                            color: color.to_string(),
                            size,
                            simulate_pressure: stroke.and_then(|s| s.get("simulatePressure")).and_then(Value::as_bool),
                            fill,
                        },
                    },
                    _ => FactData::Other { kind },
                }
            }
            "view-box" => {
                let vb = value.get("value");
                match (vb.and_then(|v| v.get("x")).and_then(Value::as_f64), vb.and_then(|v| v.get("y")).and_then(Value::as_f64), vb.and_then(|v| v.get("width")).and_then(Value::as_f64), vb.and_then(|v| v.get("height")).and_then(Value::as_f64)) {
                    (Some(x), Some(y), Some(width), Some(height)) => FactData::ViewBox { value: ViewBox { x, y, width, height } },
                    _ => FactData::Other { kind },
                }
            }
            // date-time and the *-union attributes: {value: string} (+extras).
            _ if matches!(kind.as_str(), "date-time" | "awareness" | "color") || kind.ends_with("-union") => match string_at("value") {
                Some(v) => FactData::Union { kind, value: v.to_string() },
                None => FactData::Other { kind },
            },
            _ => FactData::Other { kind },
        }
    }

    /// The payload's wire type tag (`"ordered-reference"`, …).
    pub fn kind(&self) -> &str {
        match self {
            FactData::Text { .. } => "text",
            FactData::String { .. } => "string",
            FactData::StringArray { .. } => "string-array",
            FactData::Boolean { .. } => "boolean",
            FactData::Number { .. } => "number",
            FactData::Reference { .. } => "reference",
            FactData::OrderedReference { .. } => "ordered-reference",
            FactData::SpatialReference { .. } => "spatial-reference",
            FactData::Image(_) => "image",
            FactData::BlueskyPost { .. } => "bluesky-post",
            FactData::InkStroke { .. } => "ink-stroke",
            FactData::ViewBox { .. } => "view-box",
            FactData::Union { kind, .. } => kind,
            FactData::Other { kind } => kind,
        }
    }

    /// String payload of `text`/`string`/union attributes.
    pub fn str_value(&self) -> Option<&str> {
        match self {
            FactData::Text { value } | FactData::String { value } | FactData::Union { value, .. } => Some(value),
            _ => None,
        }
    }
}

/// One fact row (the pull shape, `FactWithIndexes` in
/// `src/replicache/utils.ts`; `indexes` is recomputed client-side and
/// ignored here).
#[derive(Debug, Clone, PartialEq)]
pub struct FactRow {
    pub id: String,
    pub entity: String,
    pub attribute: String,
    pub data: FactData,
}

impl FactRow {
    /// First `text`/`string`/union value fact for the attribute, if exactly
    /// string-shaped.
    pub fn value(&self) -> Option<&str> {
        self.data.str_value()
    }
}

/// `entity → attribute → [facts]` scan index, the pull-side shape of
/// Leaflet's `scanIndexLocal`. Insertion order (wire order) is preserved.
#[derive(Debug, Clone, Default)]
pub struct ScanIndex {
    map: HashMap<(String, String), Vec<FactRow>>,
}

impl ScanIndex {
    pub fn from_facts(rows: Vec<FactRow>) -> Self {
        let mut map: HashMap<(String, String), Vec<FactRow>> = HashMap::new();
        for row in rows {
            map.entry((row.entity.clone(), row.attribute.clone())).or_default().push(row);
        }
        Self { map }
    }

    /// All facts for `(entity, attribute)`, wire order.
    pub fn eav(&self, entity: &str, attribute: &str) -> &[FactRow] {
        self.map.get(&(entity.to_string(), attribute.to_string())).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Cardinality-`one` read: the first fact, if any.
    pub fn one(&self, entity: &str, attribute: &str) -> Option<&FactRow> {
        self.eav(entity, attribute).first()
    }

    /// Cardinality-`one` string value (`text`/`string`/union payloads).
    pub fn str(&self, entity: &str, attribute: &str) -> Option<&str> {
        self.one(entity, attribute).and_then(|f| f.data.str_value())
    }

    /// Cardinality-`one` boolean value.
    pub fn boolean(&self, entity: &str, attribute: &str) -> Option<bool> {
        match self.one(entity, attribute).map(|f| &f.data) {
            Some(FactData::Boolean { value }) => Some(*value),
            _ => None,
        }
    }

    /// Cardinality-`one` number value.
    pub fn number(&self, entity: &str, attribute: &str) -> Option<f64> {
        match self.one(entity, attribute).map(|f| &f.data) {
            Some(FactData::Number { value }) => Some(*value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_every_spine_payload_shape() {
        let cases: Vec<serde_json::Value> = vec![
            json!({"type": "text", "value": "AQ=="}),
            json!({"type": "string", "value": "plain"}),
            json!({"type": "string-array", "value": ["a", "b"]}),
            json!({"type": "boolean", "value": true}),
            json!({"type": "number", "value": 2}),
            json!({"type": "number", "value": 1.5}),
            json!({"type": "reference", "value": "e2"}),
            json!({"type": "ordered-reference", "value": "e2", "position": "a0"}),
            json!({"type": "spatial-reference", "value": "e2", "position": {"x": 1, "y": 2.5}}),
            json!({"type": "image", "src": "https://x/img.png", "fallback": "f", "height": 100, "width": 50, "local": null}),
            json!({"type": "bluesky-post", "value": {"post": {"uri": "at://x", "cid": "b"}}}),
            json!({"type": "ink-stroke", "value": {"points": [1, 2, 3], "color": "#fff", "size": 4, "simulatePressure": true, "fill": {"points": [1, 2, 3, 4], "color": "#000"}}}),
            json!({"type": "view-box", "value": {"x": 0, "y": 0, "width": 10.5, "height": 9}}),
            json!({"type": "block-type-union", "value": "heading"}),
            json!({"type": "date-time", "value": "2026-01-01T00:00:00Z", "originalTimezone": "Z"}),
            json!({"type": "awareness", "value": "someone"}),
            json!({"type": "color", "value": "1,0.5,0,1"}),
        ];
        let parsed: Vec<FactData> = cases.iter().map(|v| FactData::parse(v)).collect();
        assert_eq!(parsed[0], FactData::Text { value: "AQ==".into() });
        assert_eq!(parsed[1], FactData::String { value: "plain".into() });
        assert_eq!(parsed[2], FactData::StringArray { value: vec!["a".into(), "b".into()] });
        assert_eq!(parsed[3], FactData::Boolean { value: true });
        assert_eq!(parsed[4], FactData::Number { value: 2.0 });
        assert_eq!(parsed[5], FactData::Number { value: 1.5 });
        assert_eq!(parsed[6], FactData::Reference { value: "e2".into() });
        assert_eq!(parsed[7], FactData::OrderedReference { value: "e2".into(), position: "a0".into() });
        assert_eq!(parsed[8], FactData::SpatialReference { value: "e2".into(), x: 1.0, y: 2.5 });
        assert_eq!(
            parsed[9],
            FactData::Image(ImageData { src: "https://x/img.png".into(), height: 100.0, width: 50.0 })
        );
        assert!(matches!(parsed[10], FactData::BlueskyPost { .. }));
        assert_eq!(
            parsed[11],
            FactData::InkStroke {
                value: InkStroke {
                    points: vec![1.0, 2.0, 3.0],
                    color: "#fff".into(),
                    size: 4.0,
                    simulate_pressure: Some(true),
                    fill: Some(StrokeFill { points: vec![1.0, 2.0, 3.0, 4.0], color: "#000".into() }),
                },
            }
        );
        assert_eq!(parsed[12], FactData::ViewBox { value: ViewBox { x: 0.0, y: 0.0, width: 10.5, height: 9.0 } });
        assert_eq!(parsed[13], FactData::Union { kind: "block-type-union".into(), value: "heading".into() });
        assert_eq!(parsed[14], FactData::Union { kind: "date-time".into(), value: "2026-01-01T00:00:00Z".into() });
        assert_eq!(parsed[15], FactData::Union { kind: "awareness".into(), value: "someone".into() });
        assert_eq!(parsed[16], FactData::Union { kind: "color".into(), value: "1,0.5,0,1".into() });
    }

    #[test]
    fn unknown_and_malformed_payloads_degrade_to_other() {
        assert_eq!(FactData::parse(&json!({"type": "martian", "x": 1})), FactData::Other { kind: "martian".into() });
        assert_eq!(FactData::parse(&json!({})), FactData::Other { kind: String::new() });
        // Known tag, missing members: tolerated as Other, never an error.
        assert_eq!(FactData::parse(&json!({"type": "number"})), FactData::Other { kind: "number".into() });
        assert_eq!(FactData::parse(&json!({"type": "ordered-reference", "value": "e"})), FactData::Other { kind: "ordered-reference".into() });
    }

    #[test]
    fn scan_index_reads_preserve_wire_order() {
        let rows = vec![
            row("f1", "page", "card/block", json!({"type": "ordered-reference", "value": "b1", "position": "a0"})),
            row("f2", "page", "card/block", json!({"type": "ordered-reference", "value": "b2", "position": "a1"})),
            row("f3", "b1", "block/type", json!({"type": "block-type-union", "value": "text"})),
            row("f4", "b1", "block/text", json!({"type": "text", "value": "AQ=="})),
            row("f5", "b1", "block/is-list", json!({"type": "boolean", "value": false})),
        ];
        let scan = ScanIndex::from_facts(rows);
        let ids: Vec<&str> = scan.eav("page", "card/block").iter().map(|f| f.id.as_str()).collect();
        assert_eq!(ids, vec!["f1", "f2"]);
        assert_eq!(scan.one("b1", "block/type").unwrap().value(), Some("text"));
        assert_eq!(scan.str("b1", "block/text"), Some("AQ=="));
        assert_eq!(scan.boolean("b1", "block/is-list"), Some(false));
        assert_eq!(scan.number("b1", "block/heading-level"), None);
        assert!(scan.eav("ghost", "card/block").is_empty());
    }

    fn row(id: &str, entity: &str, attribute: &str, data: serde_json::Value) -> FactRow {
        FactRow { id: id.into(), entity: entity.into(), attribute: attribute.into(), data: FactData::parse(&data) }
    }
}
