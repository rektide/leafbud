//! Yrs read path promoted from experiments 05/13: legacy schema handling,
//! complete causal integration, strict mark/XML validation, no hand-rolled codec.
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use leafspring_document::{Inline, Mark, push_run};
use serde_json::{Value, json};
use yrs::{Any, Doc, Map, Out, ReadTxn, Text, Transact, Update, Xml, XmlFragment, XmlOut, types::text::YChange, updates::decoder::Decode};

pub fn read(update: &str) -> Result<Vec<Inline>> {
    let doc = Doc::new();
    doc.get_or_insert_xml_fragment("prosemirror");
    doc.transact_mut()
        .apply_update(Update::decode_v1(&STANDARD.decode(update).context("invalid base64 text")?).context("invalid Yjs v1 text")?)?;
    let txn = doc.transact();
    ensure!(
        txn.store().pending_update().is_none() && txn.store().pending_ds().is_none(),
        "incomplete Yjs state"
    );
    if let Some(meta) = txn.get_map("meta")
        && let Some(v) = meta.get(&txn, "schemaVersion")
    {
        let Out::Any(v) = v else {
            bail!("non-scalar text schema version");
        };
        ensure!(serde_json::to_value(v)?.as_u64().is_some_and(|v| v <= 1), "unsupported text schema version");
    }
    let root = txn.get_xml_fragment("prosemirror").context("missing prosemirror")?;
    ensure!(root.len(&txn) == 1, "expected exactly one text paragraph");
    let Some(XmlOut::Element(p)) = root.get(&txn, 0) else {
        bail!("missing text paragraph");
    };
    ensure!(
        p.tag().as_ref() == "paragraph" && p.attributes(&txn).next().is_none(),
        "unsupported text paragraph"
    );
    let mut out = vec![];
    for child in p.children(&txn) {
        match child {
            XmlOut::Text(text) => {
                ensure!(text.attributes(&txn).next().is_none(), "unsupported text attributes");
                for run in text.diff(&txn, YChange::identity) {
                    let Out::Any(Any::String(value)) = run.insert else {
                        bail!("non-string inline content");
                    };
                    let mut marks = vec![];
                    for (name, data) in run.attributes.as_deref().into_iter().flatten() {
                        let data: Value = serde_json::to_value(data)?;
                        let mark = match name.as_ref() {
                            "strong" | "em" | "code" | "strikethrough" | "underline" => {
                                ensure!(data == json!({}) || data == json!(true), "unexpected attributes on {name} mark");
                                match name.as_ref() {
                                    "strong" => Mark::Strong,
                                    "em" => Mark::Em,
                                    "code" => Mark::Code,
                                    "strikethrough" => Mark::Strikethrough,
                                    _ => Mark::Underline,
                                }
                            }
                            "link" => {
                                ensure!(data.as_object().is_some_and(|o| o.len() == 1), "unknown link fields");
                                Mark::Link {
                                    href: data["href"].as_str().context("missing link href")?.into(),
                                }
                            }
                            "highlight" => {
                                ensure!(data.as_object().is_some_and(|o| o.len() == 1), "unknown highlight fields");
                                Mark::Highlight {
                                    color: data["color"].as_str().context("missing highlight slot")?.into(),
                                }
                            }
                            other => bail!("unsupported text mark: {other}"),
                        };
                        marks.push(mark);
                    }
                    push_run(&mut out, &value, &marks);
                }
            }
            XmlOut::Element(e) if e.tag().as_ref() == "hard_break" && e.len(&txn) == 0 && e.attributes(&txn).next().is_none() => out.push(Inline::HardBreak),
            _ => bail!("unsupported inline XML node"),
        }
    }
    Ok(out)
}
