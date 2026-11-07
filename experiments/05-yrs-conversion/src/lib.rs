//! Offline experiment, deliberately restricted to one paragraph and text/break children.
//! No application code or transport dependency. JSON requests/results flow on stdin/stdout.
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use yrs::types::Attrs;
use yrs::types::text::YChange;
use yrs::updates::decoder::Decode;
use yrs::{
    Any, Doc, Map, OffsetKind, Options, Out, ReadTxn, StateVector, Text, Transact, TransactionMut, Update, Xml, XmlElementPrelim, XmlElementRef, XmlFragment,
    XmlOut, XmlTextPrelim, XmlTextRef,
};

const PREFIX: &str = "pub.leaflet.richtext.facet#";
const KEYS: [&str; 7] = ["strikethrough", "code", "highlight", "underline", "strong", "em", "link"];
type Marks = BTreeMap<String, Value>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Canonical {
    pub plaintext: String,
    #[serde(default)]
    pub facets: Vec<Facet>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facet {
    pub index: Span,
    pub features: Vec<Value>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Span {
    pub byte_start: usize,
    pub byte_end: usize,
}
#[derive(Clone, Debug)]
struct Scalar {
    ch: char,
    marks: Marks,
}

fn any(value: &Value) -> Result<Any> {
    Ok(serde_json::from_value(value.clone())?)
}
fn attrs(marks: &Marks) -> Result<Attrs> {
    marks.iter().map(|(k, v)| Ok((k.clone().into(), any(v)?))).collect()
}
fn feature(k: &str, v: &Value) -> Result<Value> {
    let name = match k {
        "strong" => "bold",
        "em" => "italic",
        other => other,
    };
    let mut f = json!({"$type": format!("{PREFIX}{name}")});
    match k {
        "link" => {
            let href = v["href"].as_str().context("link requires href")?;
            ensure!(v.as_object().is_some_and(|m| m.len() == 1), "unsupported link attributes");
            f["uri"] = json!(href);
        }
        "highlight" => ensure!(*v == json!({"color":"1"}), "experiment only supports highlight slot 1"),
        _ => ensure!(*v == json!({}) || *v == json!(true), "unsupported boolean-mark value: {k}={v}"),
    }
    Ok(f)
}
fn features(marks: &Marks) -> Result<Vec<Value>> {
    KEYS.iter().filter_map(|k| marks.get(*k).map(|v| feature(k, v))).collect()
}
fn canonical(scalars: &[Scalar]) -> Result<Canonical> {
    let mut out = Canonical {
        plaintext: String::new(),
        facets: vec![],
    };
    for s in scalars {
        let start = out.plaintext.len();
        out.plaintext.push(s.ch);
        let end = out.plaintext.len();
        let fs = features(&s.marks)?;
        if fs.is_empty() {
            continue;
        }
        if let Some(last) = out.facets.last_mut()
            && last.index.byte_end == start
            && last.features == fs
        {
            last.index.byte_end = end;
        } else {
            out.facets.push(Facet {
                index: Span {
                    byte_start: start,
                    byte_end: end,
                },
                features: fs,
            });
        }
    }
    Ok(out)
}
fn desired_scalars(input: &Canonical) -> Result<Vec<Scalar>> {
    let text = &input.plaintext;
    let mut spans = vec![];
    for facet in &input.facets {
        let (a, b) = (facet.index.byte_start, facet.index.byte_end);
        ensure!(
            a < b && b <= text.len() && text.is_char_boundary(a) && text.is_char_boundary(b),
            "invalid UTF-8 facet boundary {a}..{b}"
        );
        let mut marks = Marks::new();
        for f in &facet.features {
            let name = f["$type"].as_str().and_then(|v| v.strip_prefix(PREFIX)).context("invalid feature type")?;
            let (key, value) = match name {
                "bold" => ("strong", json!({})),
                "italic" => ("em", json!({})),
                "code" | "underline" | "strikethrough" => (name, json!({})),
                "highlight" => {
                    ensure!(f.get("color").is_none(), "colored highlight requires theme mapping");
                    ("highlight", json!({"color":"1"}))
                }
                "link" => ("link", json!({"href":f["uri"].as_str().context("link requires uri")?})),
                _ => bail!("unsupported facet {name}"),
            };
            let allowed = if key == "link" { 2 } else { 1 };
            ensure!(f.as_object().is_some_and(|v| v.len() == allowed), "unsupported feature fields");
            if let Some(old) = marks.insert(key.into(), value.clone()) {
                ensure!(old == value, "conflicting feature values");
            }
        }
        spans.push((a, b, marks));
    }
    text.char_indices()
        .map(|(pos, ch)| {
            let mut marks = Marks::new();
            for (a, b, m) in &spans {
                if *a <= pos && pos < *b {
                    for (k, v) in m {
                        if let Some(old) = marks.insert(k.clone(), v.clone()) {
                            ensure!(old == *v, "conflicting overlapping {k} facets");
                        }
                    }
                }
            }
            ensure!(ch != '\n' || marks.is_empty(), "marks on hard breaks are not representable by this experiment");
            Ok(Scalar { ch, marks })
        })
        .collect()
}

pub fn load(value: &str, client: u64) -> Result<Doc> {
    ensure!(client < (1u64 << 53), "client id exceeds JS safe integer range");
    let doc = Doc::with_options(Options {
        client_id: yrs::block::ClientID::new(client),
        offset_kind: OffsetKind::Utf16,
        ..Options::default()
    });
    let update = Update::decode_v1(&STANDARD.decode(value).context("base64")?).context("Yjs v1 decode")?;
    doc.transact_mut().apply_update(update).context("Yjs integration")?;
    {
        let txn = doc.transact();
        ensure!(
            txn.store().pending_update().is_none() && txn.store().pending_ds().is_none(),
            "incomplete update: missing causal state"
        );
        ensure!(txn.state_vector().get(&doc.client_id()) == 0, "client id already present in pulled state");
    }
    Ok(doc)
}
fn paragraph<T: ReadTxn>(txn: &T) -> Result<XmlElementRef> {
    let root = txn.get_xml_fragment("prosemirror").context("missing prosemirror root")?;
    ensure!(root.len(txn) == 1, "expected exactly one paragraph");
    let XmlOut::Element(p) = root.get(txn, 0).context("missing paragraph")? else {
        bail!("root child is not XML element")
    };
    ensure!(p.tag().as_ref() == "paragraph" && p.attributes(txn).next().is_none(), "unsupported paragraph");
    if let Some(meta) = txn.get_map("meta")
        && let Some(v) = meta.get(txn, "schemaVersion")
    {
        let Out::Any(v) = v else { bail!("non-scalar schema version") };
        let v: Value = serde_json::to_value(v)?;
        ensure!(v.as_u64().is_some_and(|v| v <= 1), "unsupported schema version {v}");
    }
    Ok(p)
}
fn text_scalars<T: ReadTxn>(t: &XmlTextRef, txn: &T) -> Result<Vec<Scalar>> {
    ensure!(t.attributes(txn).next().is_none(), "unsupported XML text attributes");
    let mut result = vec![];
    for run in t.diff(txn, YChange::identity) {
        let Out::Any(Any::String(s)) = run.insert else {
            bail!("unsupported embedded content in XML text")
        };
        let mut marks = Marks::new();
        for (k, v) in run.attributes.as_deref().into_iter().flatten() {
            if k.as_ref() == "comment" {
                continue;
            } // preserved in CRDT; not owned by JSON facets
            ensure!(KEYS.contains(&k.as_ref()), "unsupported mark {k}");
            let v = serde_json::to_value(v)?;
            feature(k, &v)?;
            // Normalize equivalent historical true / ProseMirror empty-object marks.
            let v = if k.as_ref() != "link" && k.as_ref() != "highlight" { json!({}) } else { v };
            marks.insert(k.to_string(), v);
        }
        result.extend(s.chars().map(|ch| Scalar { ch, marks: marks.clone() }));
    }
    Ok(result)
}
fn read<T: ReadTxn>(txn: &T) -> Result<Canonical> {
    let p = paragraph(txn)?;
    let mut scalars = vec![];
    for child in p.children(txn) {
        match child {
            XmlOut::Text(t) => scalars.extend(text_scalars(&t, txn)?),
            XmlOut::Element(e) if e.tag().as_ref() == "hard_break" && e.len(txn) == 0 && e.attributes(txn).next().is_none() => {
                scalars.push(Scalar { ch: '\n', marks: Marks::new() })
            }
            _ => bail!("unsupported inline XML node"),
        }
    }
    canonical(&scalars)
}

#[derive(Clone, Copy, Debug)]
enum Edit {
    Keep(usize, usize),
    Delete(usize),
    Insert(usize),
}
// Bounded O(n*m) LCS is intentionally experimental, with deterministic tie breaking.
fn diff<A, B>(a: &[A], b: &[B], eq: impl Fn(&A, &B) -> bool) -> Result<Vec<Edit>> {
    ensure!((a.len() + 1).saturating_mul(b.len() + 1) <= 4_000_000, "experiment LCS size limit");
    let mut dp = vec![vec![0; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            dp[i][j] = if eq(&a[i], &b[j]) {
                1 + dp[i + 1][j + 1]
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = vec![];
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && eq(&a[i], &b[j]) {
            out.push(Edit::Keep(i, j));
            i += 1;
            j += 1;
        } else if i < a.len() && (j == b.len() || dp[i + 1][j] >= dp[i][j + 1]) {
            out.push(Edit::Delete(i));
            i += 1;
        } else {
            out.push(Edit::Insert(j));
            j += 1;
        }
    }
    Ok(out)
}
fn edit_text(t: &XmlTextRef, txn: &mut TransactionMut, target: &[Scalar]) -> Result<()> {
    let old = text_scalars(t, txn)?;
    let mut pos = 0;
    for edit in diff(&old, target, |a, b| a.ch == b.ch)? {
        match edit {
            Edit::Keep(i, j) => {
                let a = &old[i];
                let b = &target[j];
                let len = a.ch.len_utf16() as u32;
                let mut delta = Attrs::new();
                for k in KEYS {
                    if a.marks.get(k) != b.marks.get(k) {
                        delta.insert(k.into(), b.marks.get(k).map(any).transpose()?.unwrap_or(Any::Null));
                    }
                }
                if !delta.is_empty() {
                    t.format(txn, pos, len, delta);
                }
                pos += len;
            }
            Edit::Delete(i) => t.remove_range(txn, pos, old[i].ch.len_utf16() as u32),
            Edit::Insert(j) => {
                let b = &target[j];
                t.insert_with_attributes(txn, pos, &b.ch.to_string(), attrs(&b.marks)?);
                pos += b.ch.len_utf16() as u32;
            }
        }
    }
    Ok(())
}
enum Segment {
    Text(Vec<Scalar>),
    Break,
}
fn segments(scalars: Vec<Scalar>) -> Vec<Segment> {
    let mut out = vec![];
    let mut text = vec![];
    for s in scalars {
        if s.ch == '\n' {
            if !text.is_empty() {
                out.push(Segment::Text(std::mem::take(&mut text)));
            }
            out.push(Segment::Break);
        } else {
            text.push(s);
        }
    }
    if !text.is_empty() {
        out.push(Segment::Text(text));
    }
    out
}
pub fn edit(doc: &Doc, target: &Canonical) -> Result<Option<Vec<u8>>> {
    let wanted = desired_scalars(target)?;
    let normalized = canonical(&wanted)?;
    {
        let txn = doc.transact();
        if read(&txn)? == normalized {
            return Ok(None);
        }
    }
    let mut txn = doc.transact_mut();
    let p = paragraph(&txn)?;
    let old: Vec<_> = p.children(&txn).collect();
    let target = segments(wanted);
    let mut pos = 0;
    for op in diff(&old, &target, |a, b| {
        matches!((a, b), (XmlOut::Text(_), Segment::Text(_)) | (XmlOut::Element(_), Segment::Break))
    })? {
        match op {
            Edit::Keep(i, j) => {
                if let (XmlOut::Text(t), Segment::Text(s)) = (&old[i], &target[j]) {
                    edit_text(t, &mut txn, s)?;
                }
                pos += 1;
            }
            Edit::Delete(_) => p.remove(&mut txn, pos),
            Edit::Insert(j) => {
                match &target[j] {
                    Segment::Break => {
                        p.insert(&mut txn, pos, XmlElementPrelim::empty("hard_break"));
                    }
                    Segment::Text(s) => {
                        let t = p.insert(&mut txn, pos, XmlTextPrelim::new(""));
                        edit_text(&t, &mut txn, s)?;
                    }
                }
                pos += 1;
            }
        }
    }
    ensure!(read(&txn)? == normalized, "internal edit/read-back mismatch");
    // Only this transaction, separate from the initial apply transaction: includes delete-only edits.
    let update = txn.encode_update_v1();
    txn.commit();
    Ok(Some(update))
}
fn observation(doc: &Doc) -> Result<Value> {
    let txn = doc.transact();
    let p = paragraph(&txn)?;
    let structure: Vec<_> = p
        .children(&txn)
        .map(|n| match n {
            XmlOut::Text(t) => json!({"type":"XmlText","utf16Length":t.len(&txn)}),
            XmlOut::Element(e) => json!({"type":"XmlElement","tag":e.tag().as_ref()}),
            _ => json!({"type":"unsupported"}),
        })
        .collect();
    let mut roots: Vec<_> = txn.root_refs().map(|(k, _)| k.to_string()).collect();
    roots.sort();
    Ok(json!({"canonical":read(&txn)?,"roots":roots,"children":structure,
        "fullUpdate":STANDARD.encode(txn.encode_state_as_update_v1(&StateVector::default()))}))
}
pub fn request(input: Value) -> Result<Value> {
    let client = input.get("client").and_then(Value::as_u64).unwrap_or(9001);
    let doc = load(input["update"].as_str().context("update is required")?, client)?;
    let before = observation(&doc)?;
    let delta = if let Some(target) = input.get("desired") {
        edit(&doc, &serde_json::from_value(target.clone())?)?
    } else {
        None
    };
    let mut out = observation(&doc)?;
    out["before"] = before;
    out["incrementalUpdate"] = delta.map(|v| json!(STANDARD.encode(v))).unwrap_or(Value::Null);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offsets_and_invalid_json() {
        let s = "é😀e\u{301}";
        assert_eq!(s.len(), 9);
        assert_eq!(s.encode_utf16().count(), 5);
        let valid = Canonical {
            plaintext: s.into(),
            facets: vec![Facet {
                index: Span { byte_start: 2, byte_end: 6 },
                features: vec![json!({"$type":format!("{PREFIX}bold")})],
            }],
        };
        assert_eq!(canonical(&desired_scalars(&valid).unwrap()).unwrap(), valid);
        let mut invalid = valid;
        invalid.facets[0].index.byte_start = 3;
        assert!(desired_scalars(&invalid).is_err());
        assert!(serde_json::from_str::<Canonical>(r#"{"plaintext":"\ud800"}"#).is_err());
    }
    #[test]
    fn conflicts_and_limits() {
        let input: Canonical = serde_json::from_value(json!({"plaintext":"a","facets":[
            {"index":{"byteStart":0,"byteEnd":1},"features":[{"$type":format!("{PREFIX}link"),"uri":"https://a.example"}]},
            {"index":{"byteStart":0,"byteEnd":1},"features":[{"$type":format!("{PREFIX}link"),"uri":"https://b.example"}]}
        ]}))
        .unwrap();
        assert!(desired_scalars(&input).is_err());
        assert!(diff(&vec![0; 3000], &vec![0; 3000], |a, b| a == b).is_err());
    }
}
