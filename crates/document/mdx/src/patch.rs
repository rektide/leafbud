//! Source-preserving envelope and author-field patching. Two distinct
//! operations: [`patch_tracking`] settles only the reserved `@leafbud`
//! envelope (identity/binding/nodes/receipts) so successful pushes never
//! reformat the author's body; [`adopt_remote`] additionally adopts changed
//! remote content and owned author fields for guarded refresh.
//! [`prepare_binding_defaults`] is separate pre-content-write preparation.
use super::{json, parse, yaml};
use anyhow::{Context, Result, bail, ensure};
use leafbud_document::*;
use serde_json::{Value, json};

/// Split a managed source into its frontmatter lines (fences excluded) and
/// the exact body text following the closing fence. The MDX frontmatter
/// block ends at the first `---` line, matching the native parser's view.
/// Both `\n` and `\r\n` fences are accepted; lines keep their own bytes.
pub(crate) fn split_managed(source: &str) -> Result<(Vec<String>, String)> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    ensure!(
        source.starts_with("---\n") || source.starts_with("---\r\n"),
        "managed frontmatter opening fence missing"
    );
    let opening = if source.starts_with("---\r\n") { "---\r\n" } else { "---\n" };
    let mut lines = Vec::new();
    let mut cursor = opening.len();
    loop {
        let Some(nl) = source[cursor..].find('\n') else {
            bail!("managed frontmatter closing fence missing");
        };
        let line = &source[cursor..cursor + nl + 1];
        if line == "---\n" || line == "---\r\n" {
            return Ok((lines, source[cursor + line.len()..].to_owned()));
        }
        lines.push(line.to_owned());
        cursor += line.len();
    }
}

/// Top-level mapping key of a column-0 line, quoted or plain. Comment lines,
/// nested content and flow continuations (always indented in valid block
/// mappings) yield None; a multiline scalar therefore cannot impersonate a key.
fn column_key(line: &str) -> Option<&str> {
    let line = line.strip_suffix('\n').unwrap_or(line);
    if line.is_empty() || line.starts_with([' ', '\t', '#']) {
        return None;
    }
    if let Some(q) = line.chars().next().filter(|c| *c == '"' || *c == '\'') {
        let close = line[1..].find(q)? + 1;
        let name = &line[1..close];
        line[close + 1..].trim_start().starts_with(':').then_some(name)
    } else {
        let colon = line.find(':')?;
        let name = &line[..colon].trim_end();
        (!name.is_empty() && !name.starts_with('[') && !name.starts_with('{')).then_some(name)
    }
}

/// Locate the single column-0 definition of `key`.
fn find_key(lines: &[String], key: &str) -> Result<Option<usize>> {
    let mut found = None;
    for (index, line) in lines.iter().enumerate() {
        if column_key(line) == Some(key) {
            ensure!(found.replace(index).is_none(), "duplicate frontmatter key {key:?} cannot be patched");
        }
    }
    Ok(found)
}

/// Exclusive end of a key's value extent: the key line plus following indented
/// or blank lines, with trailing blank separators excluded.
fn value_extent(lines: &[String], key_line: usize) -> usize {
    let mut end = key_line + 1;
    while end < lines.len() && (lines[end].starts_with([' ', '\t']) || lines[end].trim().is_empty()) {
        end += 1;
    }
    while end > key_line + 1 && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    end
}

fn key_lines(key: &str, value: &Value) -> Vec<String> {
    let mut out = String::new();
    yaml(&json!({ key.to_owned(): value }), 0, &mut out);
    out.split_inclusive('\n').map(str::to_owned).collect()
}

/// Replace, insert or remove one owned top-level key. `Patch::Remove` deletes
/// the extent; `Patch::Set` rewrites/inserts normalized lines (including an
/// explicit null spelling, which is distinct from removal).
enum Patch {
    Remove,
    Set(Value),
}

fn patch_key(lines: &mut Vec<String>, key: &str, patch: &Patch) -> Result<()> {
    match patch {
        Patch::Remove => {
            if let Some(index) = find_key(lines, key)? {
                let end = value_extent(lines, index);
                lines.splice(index..end, []);
            }
        }
        Patch::Set(value) => match find_key(lines, key)? {
            Some(index) => {
                let end = value_extent(lines, index);
                lines.splice(index..end, key_lines(key, value));
            }
            None => {
                let at = find_key(lines, "@leafbud")?.context("reserved @leafbud envelope not found for source-preserving patch")?;
                lines.splice(at..at, key_lines(key, value));
            }
        },
    }
    Ok(())
}

/// Byte range of the value following a column-0 mapping key's colon on its
/// own line. Returns None when the line carries no value (block style: the
/// value, if any, lives on following indented lines). A trailing comment is
/// not part of the value range.
fn line_value_range(line: &str) -> Option<(usize, usize)> {
    let body = line.trim_end_matches(['\r', '\n']);
    let first = body.chars().next()?;
    let colon = if first == '"' || first == '\'' {
        let close = body[1..].find(first)? + 1;
        body[close + 1..].find(':')? + close + 1
    } else {
        body.find(':')?
    };
    let rest = &body[colon + 1..];
    let trimmed = rest.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    Some((body.len() - trimmed.len(), body.len()))
}

/// Persist one mapping entry in the reserved `@leafbud` envelope (used for
/// main-page identity during preparation). Envelope shape is detected from
/// the envelope line's own syntax, not its surroundings: a block-style
/// envelope keeps every other line byte-for-byte and the entry joins at the
/// children's measured indentation; a flow-style envelope is edited as the
/// literal JSON mapping it must be, preserving its flow spelling. No full
/// document parse happens here; callers prove the composed result.
pub(crate) fn insert_state_line(source: &str, line: &str) -> Result<String> {
    let (key, value) = line.split_once(':').context("state line must be a mapping entry")?;
    let parsed_value: Value = serde_json::from_str(value.trim()).with_context(|| format!("parsing prepared state line {line:?}"))?;
    ensure!(key == "\"mainPage\"", "preparation does not own state field {key}");
    let (lines, body) = split_managed(source)?;
    let envelope = find_key(&lines, "@leafbud")?.context("reserved @leafbud envelope not found for preparation")?;
    if lines.iter().any(|l| column_key(l).is_some_and(|k| k == key)) {
        bail!("frontmatter key {key:?} already exists elsewhere; refusing nested insertion");
    }
    let newline = if source.contains("\r\n") { "\r\n" } else { "\n" };
    match line_value_range(&lines[envelope]) {
        // Flow style: edit the literal JSON mapping in place, inserting the
        // new member before the closing brace with the mapping's own
        // separator style, so the author's line keeps its bytes otherwise.
        Some((start, end)) => {
            let line = lines[envelope].clone();
            let segment = &line[start..end];
            let raw = segment.trim();
            let mut flow: Value = serde_json::from_str(raw).with_context(|| "reserved envelope is flow-style but its value is not literal JSON")?;
            let Some(map) = flow.as_object_mut() else {
                bail!("reserved envelope is flow-style but its value is not a mapping");
            };
            let name: String = serde_json::from_str(key).context("state line key")?;
            let member_value = json(&parsed_value);
            ensure!(map.insert(name, parsed_value).is_none(), "state key {key:?} already exists in the envelope");
            ensure!(raw.ends_with('}'), "the flow envelope mapping does not end on its own line");
            let (separator, colon) = if segment.contains("\": ") { (", ", ": ") } else { (",", ":") };
            let inner = &raw[..raw.len() - 1];
            let member = format!("{}{key}{colon}{member_value}", if inner.trim_end().ends_with('{') { "" } else { separator },);
            let brace_at = start + segment.trim_end().len() - 1;
            let mut updated = line[..brace_at].to_owned();
            updated.push_str(&member);
            updated.push_str(&line[brace_at..]);
            let mut lines = lines;
            lines[envelope] = updated;
            Ok(reassemble(source, &lines, &body))
        }
        // Block style: splice one indented line at the children's measured
        // indentation, keeping every existing line byte-for-byte.
        None => {
            let extent = value_extent(&lines, envelope);
            let child = lines[envelope + 1..extent].iter().find(|l| !l.trim().is_empty());
            let indent = child.map(|l| l.len() - l.trim_start().len()).unwrap_or(2);
            if child.is_some_and(|l| l.trim_start().starts_with('\t')) {
                bail!("envelope children use tab indentation; refusing to mix indentation");
            }
            for l in &lines[envelope..extent] {
                if l.trim_start().starts_with(&format!("{key}:")) {
                    bail!("state key {key:?} already exists in the envelope");
                }
            }
            let mut lines = lines;
            lines.splice(envelope + 1..envelope + 1, vec![format!("{}{line}{newline}", " ".repeat(indent))]);
            Ok(reassemble(source, &lines, &body))
        }
    }
}

/// Rebuild a managed source from frontmatter lines and the body, keeping the
/// original's BOM presence and fence line-ending style.
fn reassemble(source: &str, lines: &[String], body: &str) -> String {
    let bom = if source.starts_with('\u{feff}') {
        "\u{feff}".to_string()
    } else {
        String::new()
    };
    let stripped = source.strip_prefix('\u{feff}').unwrap_or(source);
    let fence = if stripped.starts_with("---\r\n") { "---\r\n" } else { "---\n" };
    let mut result = format!("{bom}{fence}");
    result.push_str(&lines.join(""));
    result.push_str(fence);
    result.push_str(body);
    result
}

/// Regenerate the reserved `@leafbud` envelope in-place, preserving every
/// other frontmatter line and the body bytes.
fn replace_envelope(source: &str, state: &State) -> Result<String> {
    let (mut lines, body) = split_managed(source)?;
    let envelope = find_key(&lines, "@leafbud")?.context("reserved @leafbud envelope not found for source-preserving patch")?;
    let end = value_extent(&lines, envelope);
    let mut regenerated = String::new();
    yaml(&json!({"@leafbud": state}), 0, &mut regenerated);
    lines.splice(envelope..end, regenerated.split_inclusive('\n').map(str::to_owned));
    Ok(reassemble(source, &lines, &body))
}

/// Settle tracking state only: replace the reserved envelope with `next`,
/// keeping author fields, comments, body bytes and unrelated state untouched.
///
/// Guards: the local document identity must not change; an established
/// binding is preserved whole (origin, token, root, entity set, entry fact,
/// publication association — never dropped or altered), with the deliberate
/// first binding (`None → Some`) as the only transition; main-page identity
/// is persisted by preparation, never by settlement. Nodes/receipts follow
/// the verified observation this settlement records, so the authored
/// projection (not the full semantic value, which includes that tracking)
/// must be identical before and after, together with the exact body bytes.
pub fn patch_tracking(source: &str, next: &State) -> Result<String> {
    let existing = parse(source)?;
    ensure!(existing.state.id == next.id, "tracking settlement cannot change document identity");
    ensure!(next.state == "draft", "unsupported lifecycle");
    if let Some(before) = &existing.state.writer {
        let after = next.writer.as_ref().context("tracking cannot discard a writer's reserved mutation IDs")?;
        ensure!(
            before.client_id == after.client_id && before.client_group_id == after.client_group_id && after.next_mutation_id >= before.next_mutation_id,
            "tracking cannot reset writer identity/sequence"
        );
    }
    ensure!(
        next.pending.is_none() || next.sync == existing.state.sync,
        "unresolved effects cannot advance agreement receipts"
    );
    match (&existing.state.binding, &next.binding) {
        (Some(before), Some(after)) => ensure!(before == after, "tracking settlement cannot rebind or alter an established binding"),
        (Some(_), None) => bail!("tracking settlement cannot drop an established binding"),
        (None, Some(_)) | (None, None) => {}
    }
    ensure!(
        existing.state.main_page == next.main_page,
        "tracking settlement cannot change main-page identity; preparation persists it"
    );
    // Correspondence already recorded: do not normalize merely because an
    // intentional alias edit left the map in a different textual key order.
    // All settlement guards above still apply; no receipt is advanced here.
    if existing.state == *next {
        return Ok(source.to_owned());
    }
    let result = replace_envelope(source, next)?;
    let parsed = parse(&result).context("tracking settlement validation")?;
    ensure!(parsed.state == *next, "tracking settlement diverges from the intended state");
    // Everything outside tracking must be untouched: same authored content
    // (pages/keys/metadata) as the pre-settlement document.
    ensure!(
        parsed.authored_value() == existing.authored_value(),
        "tracking settlement changed authored content"
    );
    let (_, existing_body) = split_managed(source)?;
    let (_, result_body) = split_managed(&result)?;
    ensure!(existing_body == result_body, "tracking settlement must not rewrite the body");
    Ok(result)
}

/// First-binding preparation ONLY: make the verified publication association
/// explicit and copy native theme/bylines only where the source field is absent.
/// This does not adopt seed prose, keys, tracking, title, description or tags.
/// The caller must verify the native seed/rights and previous attempt hashes,
/// then atomically persist this result AND updated mint hashes/binding. Never
/// persist the intermediate string with the old hashes or use it after a push.
pub fn prepare_binding_defaults(source: &str, verified: &Document) -> Result<String> {
    let existing = parse(source)?;
    let binding = verified.state.binding.as_ref().context("verified native binding missing")?;
    let attempt = existing.state.pending.as_ref().context("default preparation requires a located mint")?;
    ensure!(
        matches!(&attempt.effect, Effect::Mint { base_url, publication, stage: MintStage::Located { token } }
            if base_url == &binding.base_url && publication.publication() == binding.publication.as_deref() && *token == binding.token)
            && existing.state.writer.is_none()
            && existing.state.sync.is_none(),
        "defaults can only be prepared before native content writes"
    );
    ensure!(
        existing.state.binding.as_ref().is_none_or(|before| before == binding),
        "default preparation cannot rebind"
    );
    let publication = binding.publication.clone().map(Publication::At).unwrap_or(Publication::Standalone);
    ensure!(
        existing.publication == Publication::Inherit || existing.publication == publication,
        "explicit publication differs from verified binding"
    );
    let header = super::header(source)?;
    let mut expected = existing.clone();
    let (mut lines, body) = split_managed(source)?;
    if header.get("theme").is_none() {
        if let Some(theme) = &verified.theme {
            expected.theme = Some(theme.clone());
            patch_key(&mut lines, "theme", &Patch::Set(json!(theme)))?;
        }
    } else {
        ensure!(!header["theme"].is_null(), "explicit theme null has no supported native meaning");
    }
    if let Some(contributors) = header.get("contributors") {
        let _: Vec<String> =
            serde_json::from_value(contributors.clone()).context("explicit contributors must be a list; null has no supported native meaning")?;
    } else if let Some(contributors) = verified.extra.get("contributors") {
        let _: Vec<String> = serde_json::from_value(contributors.clone()).context("invalid verified contributors")?;
        expected.extra.insert("contributors".into(), contributors.clone());
        patch_key(&mut lines, "contributors", &Patch::Set(contributors.clone()))?;
    }
    if existing.publication != publication {
        patch_key(&mut lines, "publication", &Patch::Set(json!(binding.publication)))?;
        expected.publication = publication;
    }
    let result = reassemble(source, &lines, &body);
    let parsed = parse(&result).context("binding-default preparation validation")?;
    ensure!(parsed == expected, "binding-default preparation changed unrelated content/tracking");
    ensure!(split_managed(&result)?.1 == body, "binding-default preparation changed the body");
    Ok(result)
}

/// Adopt remote-owned content for guarded refresh. `document` is the reviewed
/// next state of THIS bound document (same format, id and binding token).
/// Owned author fields that changed are rewritten in normalized form;
/// unchanged fields, comments, blank lines and unknown/local-only fields keep
/// their bytes; the reserved `@leafbud` envelope is regenerated. The body is
/// re-serialized only when page content actually changed. The result is
/// verified to parse back to exactly `document`.
pub fn adopt_remote(source: &str, document: &Document) -> Result<String> {
    let existing = parse(source)?;
    ensure!(existing.format == document.format, "adoption cannot change the document format");
    ensure!(
        existing.state.id == document.state.id
            && existing.state.binding.as_ref().map(|b| b.token) == document.state.binding.as_ref().map(|b| b.token)
            && document.state.binding.is_some(),
        "adoption cannot change document identity"
    );
    let (mut lines, body) = split_managed(source)?;
    for (key, old, new) in [
        ("title", json!(existing.title), json!(document.title)),
        (
            "description",
            json!(canonical_description(existing.description.as_deref())),
            json!(canonical_description(document.description.as_deref())),
        ),
        ("tags", json!(existing.tags), json!(document.tags)),
        ("theme", json!(existing.theme), json!(document.theme)),
        (
            "contributors",
            json!(existing.extra.get("contributors")),
            json!(document.extra.get("contributors")),
        ),
    ] {
        if old == new {
            continue;
        }
        let patch = if new.is_null() { Patch::Remove } else { Patch::Set(new) };
        patch_key(&mut lines, key, &patch)?;
    }
    // Publication intent is tri-state: absent inherits, explicit null is
    // standalone, a URI targets a publication. Preserve the distinction.
    if existing.publication != document.publication {
        let patch = match &document.publication {
            Publication::Inherit => Patch::Remove,
            Publication::Standalone => Patch::Set(json!(null)),
            Publication::At(uri) => Patch::Set(json!(uri)),
        };
        patch_key(&mut lines, "publication", &patch)?;
    }
    let envelope = find_key(&lines, "@leafbud")?.context("reserved @leafbud envelope not found for source-preserving patch")?;
    let end = value_extent(&lines, envelope);
    let mut regenerated = String::new();
    yaml(&json!({"@leafbud": &document.state}), 0, &mut regenerated);
    lines.splice(envelope..end, regenerated.split_inclusive('\n').map(str::to_owned));
    let fence = if source.strip_prefix('\u{feff}').unwrap_or(source).starts_with("---\r\n") {
        "---\r\n"
    } else {
        "---\n"
    };
    let frontmatter = format!("{fence}{}{fence}", lines.join(""));
    let body = if existing.pages == document.pages {
        body
    } else {
        super::render::verified_body(document, &frontmatter)?
    };
    let bom = if source.starts_with('\u{feff}') {
        "\u{feff}".to_string()
    } else {
        String::new()
    };
    let result = bom.clone() + &frontmatter + &body;
    let parsed = parse(&result).context("source-preserving adoption validation")?;
    ensure!(
        parsed.semantic_value() == document.semantic_value() && parsed.state == document.state,
        "source-preserving adoption would lose content/associations"
    );
    Ok(result)
}
