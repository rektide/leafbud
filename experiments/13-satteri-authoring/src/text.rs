//! Fresh-only Yrs encoding, using experiment 05's verified XML/mark conventions.
//! No LCS, retained state, old-entity overwrite or incremental synchronization.
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use yrs::{Any, Doc, Map, OffsetKind, Options, Out, ReadTxn, StateVector, Text, Transact, Update, Xml, XmlElementPrelim, XmlFragment, XmlOut, XmlTextPrelim};
use yrs::{
    types::{Attrs, text::YChange},
    updates::decoder::Decode,
};

use crate::model::{Inline, Mark, push_run};

fn attributes(marks: &[Mark]) -> Result<Attrs> {
    marks
        .iter()
        .map(|mark| {
            let (name, value) = match mark {
                Mark::Strong => ("strong", json!({})),
                Mark::Em => ("em", json!({})),
                Mark::Code => ("code", json!({})),
                Mark::Strikethrough => ("strikethrough", json!({})),
                Mark::Link { href } => ("link", json!({"href": href})),
            };
            Ok((name.into(), serde_json::from_value::<Any>(value)?))
        })
        .collect()
}

pub fn fresh(content: &[Inline], client: u64) -> Result<String> {
    ensure!(client > 0 && client < 1 << 53, "client ID must be a positive JS-safe integer");
    let doc = Doc::with_options(Options {
        client_id: yrs::block::ClientID::new(client),
        offset_kind: OffsetKind::Utf16,
        ..Options::default()
    });
    let root = doc.get_or_insert_xml_fragment("prosemirror");
    let meta = doc.get_or_insert_map("meta");
    {
        let mut txn = doc.transact_mut();
        meta.insert(&mut txn, "schemaVersion", 1);
        let paragraph = root.insert(&mut txn, 0, XmlElementPrelim::empty("paragraph"));
        for (index, inline) in content.iter().enumerate() {
            match inline {
                Inline::HardBreak => {
                    paragraph.insert(&mut txn, index as u32, XmlElementPrelim::empty("hard_break"));
                }
                Inline::Run { text, marks } => {
                    let node = paragraph.insert(&mut txn, index as u32, XmlTextPrelim::new(""));
                    node.insert_with_attributes(&mut txn, 0, text, attributes(marks)?);
                }
            }
        }
    }
    let update = doc.transact().encode_state_as_update_v1(&StateVector::default());
    Ok(STANDARD.encode(update))
}

fn load(update: &str) -> Result<Doc> {
    let doc = Doc::new();
    doc.get_or_insert_xml_fragment("prosemirror");
    doc.transact_mut().apply_update(Update::decode_v1(&STANDARD.decode(update)?)?)?;
    {
        let txn = doc.transact();
        ensure!(
            txn.store().pending_update().is_none() && txn.store().pending_ds().is_none(),
            "incomplete initial update"
        );
    }
    Ok(doc)
}

pub fn read(update: &str) -> Result<Vec<Inline>> {
    let doc = load(update)?;
    let txn = doc.transact();
    let meta = txn.get_map("meta").context("missing meta")?;
    ensure!(
        meta.get(&txn, "schemaVersion").map(|v| v.to_string(&txn)).as_deref() == Some("1"),
        "wrong schema version"
    );
    let root = txn.get_xml_fragment("prosemirror").context("missing prosemirror")?;
    ensure!(root.len(&txn) == 1, "expected exactly one paragraph");
    let Some(XmlOut::Element(p)) = root.get(&txn, 0) else {
        bail!("missing paragraph")
    };
    ensure!(p.tag().as_ref() == "paragraph" && p.attributes(&txn).next().is_none(), "unsupported paragraph");
    let mut out = vec![];
    for child in p.children(&txn) {
        match child {
            XmlOut::Text(text) => {
                for run in text.diff(&txn, YChange::identity) {
                    let Out::Any(Any::String(value)) = run.insert else { bail!("non-string text") };
                    let mut marks = vec![];
                    for (name, data) in run.attributes.as_deref().into_iter().flatten() {
                        let data = serde_json::to_value(data)?;
                        let mark = match name.as_ref() {
                            "strong" | "em" | "code" | "strikethrough" => {
                                ensure!(data == json!({}), "unexpected mark attributes");
                                match name.as_ref() {
                                    "strong" => Mark::Strong,
                                    "em" => Mark::Em,
                                    "code" => Mark::Code,
                                    _ => Mark::Strikethrough,
                                }
                            }
                            "link" => {
                                ensure!(data.as_object().is_some_and(|o| o.len() == 1), "unexpected link fields");
                                Mark::Link {
                                    href: data["href"].as_str().context("missing link href")?.into(),
                                }
                            }
                            _ => bail!("unknown mark {name}"),
                        };
                        marks.push(mark);
                    }
                    push_run(&mut out, &value, &marks);
                }
            }
            XmlOut::Element(e) if e.tag().as_ref() == "hard_break" && e.len(&txn) == 0 && e.attributes(&txn).next().is_none() => out.push(Inline::HardBreak),
            _ => bail!("unsupported inline XML"),
        }
    }
    Ok(out)
}

/// Narrow server-merge control for the explicit wrong-old-entity regression test.
pub fn merge(a: &str, b: &str) -> Result<String> {
    let doc = load(a)?;
    doc.transact_mut().apply_update(Update::decode_v1(&STANDARD.decode(b)?)?)?;
    let bytes = doc.transact().encode_state_as_update_v1(&StateVector::default());
    Ok(STANDARD.encode(bytes))
}

pub fn prosemirror(content: &[Inline]) -> Value {
    let nodes: Vec<_> = content
        .iter()
        .map(|inline| match inline {
            Inline::HardBreak => json!({"type":"hard_break"}),
            Inline::Run { text, marks } => {
                let marks: Vec<_> = marks
                    .iter()
                    .map(|mark| match mark {
                        Mark::Strong => json!({"type":"strong"}),
                        Mark::Em => json!({"type":"em"}),
                        Mark::Code => json!({"type":"code"}),
                        Mark::Strikethrough => json!({"type":"strikethrough"}),
                        Mark::Link { href } => json!({"type":"link", "attrs":{"href":href}}),
                    })
                    .collect();
                if marks.is_empty() {
                    json!({"type":"text", "text":text})
                } else {
                    json!({"type":"text", "text":text, "marks":marks})
                }
            }
        })
        .collect();
    if nodes.is_empty() {
        json!({"type":"doc","content":[{"type":"paragraph"}]})
    } else {
        json!({"type":"doc","content":[{"type":"paragraph","content":nodes}]})
    }
}
