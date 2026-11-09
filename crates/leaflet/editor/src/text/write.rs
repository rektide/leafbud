//! Bounded loaded-document edits. No independent replacement of existing state.
use super::*;
use yrs::{OffsetKind, Options, StateVector, TransactionMut, XmlElementPrelim, XmlElementRef, XmlTextPrelim, XmlTextRef, types::Attrs};

const MARKS: [&str; 7] = ["strong", "em", "code", "strikethrough", "underline", "link", "highlight"];

fn attributes(marks: &[Mark]) -> Result<Attrs> {
    let mut attrs = Attrs::new();
    for mark in marks {
        let (name, value) = match mark {
            Mark::Strong => ("strong", json!({})),
            Mark::Em => ("em", json!({})),
            Mark::Code => ("code", json!({})),
            Mark::Strikethrough => ("strikethrough", json!({})),
            Mark::Underline => ("underline", json!({})),
            Mark::Link { href } => ("link", json!({"href":href})),
            Mark::Highlight { color } => ("highlight", json!({"color":color})),
        };
        let value: Any = serde_json::from_value(value)?;
        ensure!(attrs.insert(name.into(), value).is_none(), "multiple values for {name} mark");
    }
    Ok(attrs)
}

pub fn normalize(content: &[Inline]) -> Result<Vec<Inline>> {
    let mut result = Vec::new();
    for inline in content {
        match inline {
            Inline::Run { text, marks } => {
                attributes(marks)?;
                push_run(&mut result, text, marks);
            }
            Inline::HardBreak => result.push(Inline::HardBreak),
        }
    }
    Ok(result)
}

fn format(text: &XmlTextRef, txn: &mut TransactionMut<'_>, content: &[Inline]) -> Result<()> {
    let mut offset = 0;
    for inline in content {
        let Inline::Run { text: value, marks } = inline else {
            bail!("break in text segment")
        };
        let mut attrs: Attrs = MARKS.into_iter().map(|name| (name.into(), Any::Null)).collect();
        attrs.extend(attributes(marks)?);
        let len = value.encode_utf16().count() as u32;
        if len > 0 {
            text.format(txn, offset, len, attrs);
        }
        offset += len;
    }
    Ok(())
}

fn fill(paragraph: &XmlElementRef, txn: &mut TransactionMut<'_>, content: &[Inline]) -> Result<()> {
    let mut runs = Vec::new();
    let mut index = 0;
    let flush = |runs: &mut Vec<Inline>, index: &mut u32, txn: &mut TransactionMut<'_>| -> Result<()> {
        if !runs.is_empty() {
            let value: String = runs
                .iter()
                .map(|r| match r {
                    Inline::Run { text, .. } => text.as_str(),
                    _ => unreachable!(),
                })
                .collect();
            let text = paragraph.insert(txn, *index, XmlTextPrelim::new(value));
            format(&text, txn, runs)?;
            runs.clear();
            *index += 1;
        }
        Ok(())
    };
    for inline in content {
        match inline {
            Inline::HardBreak => {
                flush(&mut runs, &mut index, txn)?;
                paragraph.insert(txn, index, XmlElementPrelim::empty("hard_break"));
                index += 1;
            }
            run => runs.push(run.clone()),
        }
    }
    flush(&mut runs, &mut index, txn)
}

/// None means semantic equality, including differently segmented XML. Otherwise
/// returns one cumulative v1 payload, including the loaded state/delete set.
pub fn write(current: Option<&str>, desired: &[Inline]) -> Result<Option<String>> {
    let desired = normalize(desired)?;
    if let Some(current) = current
        && read(current)? == desired
    {
        return Ok(None);
    }
    let doc = Doc::with_options(Options {
        offset_kind: OffsetKind::Utf16,
        ..Options::default()
    });
    let root = doc.get_or_insert_xml_fragment("prosemirror");
    let meta = current.is_none().then(|| doc.get_or_insert_map("meta"));
    if let Some(current) = current {
        doc.transact_mut().apply_update(Update::decode_v1(&STANDARD.decode(current)?)?)?;
        ensure!(doc.transact().state_vector().get(&doc.client_id()) == 0, "Yrs writer ID collision");
        // read above checked causal completeness and all active semantics.
    }
    {
        let mut txn = doc.transact_mut();
        let paragraph = if current.is_some() {
            let Some(XmlOut::Element(p)) = root.get(&txn, 0) else {
                bail!("missing loaded paragraph")
            };
            p
        } else {
            meta.as_ref().expect("new document").insert(&mut txn, "schemaVersion", 1);
            root.insert(&mut txn, 0, XmlElementPrelim::empty("paragraph"))
        };
        let compatible = !desired.iter().any(|i| matches!(i, Inline::HardBreak)) && paragraph.len(&txn) == 1;
        if compatible && let Some(XmlOut::Text(text)) = paragraph.get(&txn, 0) {
            // XmlText::get_string serializes markup, not just characters.
            // Offset arithmetic must use the actual string inserts instead.
            let mut old = Vec::new();
            for run in text.diff(&txn, YChange::identity) {
                let Out::Any(Any::String(value)) = run.insert else {
                    bail!("non-string inline content")
                };
                old.extend(value.chars());
            }
            let wanted: String = desired
                .iter()
                .map(|i| match i {
                    Inline::Run { text, .. } => text.as_str(),
                    _ => unreachable!(),
                })
                .collect();
            let new: Vec<_> = wanted.chars().collect();
            let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
            let suffix = old[prefix..].iter().rev().zip(new[prefix..].iter().rev()).take_while(|(a, b)| a == b).count();
            let offset = old[..prefix].iter().map(|c| c.len_utf16() as u32).sum();
            let remove = old[prefix..old.len() - suffix].iter().map(|c| c.len_utf16() as u32).sum();
            if remove > 0 {
                text.remove_range(&mut txn, offset, remove);
            }
            let insert: String = new[prefix..new.len() - suffix].iter().collect();
            if !insert.is_empty() {
                text.insert(&mut txn, offset, &insert);
            }
            format(&text, &mut txn, &desired)?;
        } else {
            // Selected before effects, not an escalation after an edit failure.
            // The loaded paragraph and all unrelated roots survive.
            let len = paragraph.len(&txn);
            if len > 0 {
                paragraph.remove_range(&mut txn, 0, len);
            }
            fill(&paragraph, &mut txn, &desired)?;
        }
    }
    let payload = STANDARD.encode(doc.transact().encode_state_as_update_v1(&StateVector::default()));
    ensure!(read(&payload)? == desired, "edited native text differs from desired meaning");
    Ok(Some(payload))
}
