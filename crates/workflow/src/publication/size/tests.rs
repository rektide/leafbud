use super::*;
use leafbud_document::{Block, Inline, Mark, Page};
use serde_json::json;
use uuid::Uuid;

fn fixtures() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/native-publication-size.json")).unwrap()
}
fn document(fixture: &Value) -> Document {
    let facts = fixture["facts"].as_array().unwrap();
    let content = leafbud_editor::observe(facts, fixture["root"].as_str().unwrap(), Uuid::nil()).unwrap();
    let mut document: Document = serde_json::from_value(json!({"format":"leafbud-mdx/2","title":"T","publication":null,
        "@leafbud":{"id":Uuid::nil(),"state":"draft","binding":{"baseUrl":"https://leaflet.pub","token":Uuid::nil(),"root":fixture["root"],"entitySet":Uuid::nil(),"entryFact":content.entry_fact,"publication":null}}})).unwrap();
    document.state.nodes = content.nodes;
    document.state.main_page = Some(content.pages[0].key.clone());
    document.pages = content.pages;
    document
}
fn compile(desired: &Document, current: &Document, facts: &[Value]) -> Compiled {
    leafbud_editor::write::compile(desired, current, facts, Some("T"), &Default::default()).unwrap()
}

#[test]
fn pinned_actual_native_js_yjs_json_size_and_offload_boundaries() {
    let evidence = fixtures();
    assert_eq!(evidence["revision"], "600c7b5f1f7b31b66fb626c293cc104312a12caf");
    assert_eq!(evidence["fixtures"].as_array().unwrap().len(), 6);
    for fixture in evidence["fixtures"].as_array().unwrap() {
        let doc = document(fixture);
        let facts = fixture["facts"].as_array().unwrap();
        let compiled = compile(&doc, &doc, facts);
        assert!(compiled.mutations.is_empty(), "{}", fixture["name"]);
        let witness = super::compiled(&doc, facts, &compiled).unwrap();
        let actual = fixture["json_utf8_bytes"].as_u64().unwrap() as usize;
        if fixture["name"] == "inactive-text-and-absent-language" {
            assert_eq!(witness.pages_json_utf8_upper_bound, actual + ",\"language\":\"\"".len());
        } else {
            assert_eq!(witness.pages_json_utf8_upper_bound, actual, "{}", fixture["name"]);
        }
        assert_eq!(
            witness.no_offload_at_witness,
            !fixture["would_offload"].as_bool().unwrap(),
            "{}",
            fixture["name"]
        );
        assert_eq!(witness.provisional_page_ids, 0);
        assert!(witness.requires_fresh_saved_native_witness_before_dispatch);
        if fixture["name"] == "fragmented-marked" {
            // Independent native emits 1600 facets even though typed meaning
            // coalesces into ONE run. Semantic-only sizing is an unsafe proof.
            assert_eq!(witness.active_text_segments, 1600);
            assert!(leafbud_records::projection::inline_pages_json_upper_bound(&doc).unwrap() < INLINE_THRESHOLD);
            assert!(witness.pages_json_utf8_upper_bound > INLINE_THRESHOLD);
        }
    }
}

#[test]
fn compiled_cumulative_text_and_new_pages_are_sized_before_any_save() {
    let evidence = fixtures();
    let fixture = &evidence["fixtures"][0];
    let current = document(fixture);
    let facts = fixture["facts"].as_array().unwrap();
    let mut desired = current.clone();
    desired.pages[0].blocks[0].kind = BlockKind::Paragraph {
        content: vec![Inline::Run {
            text: "New shorter 🦋".into(),
            marks: vec![Mark::Strong],
        }],
    };
    desired.pages[0].blocks.push(Block {
        key: "new-card".into(),
        kind: BlockKind::PageLink { page: "new-page".into() },
    });
    desired.pages.push(Page {
        key: "new-page".into(),
        blocks: vec![Block {
            key: "new-paragraph".into(),
            kind: BlockKind::Paragraph {
                content: vec![
                    Inline::Run {
                        text: "A new paragraph".into(),
                        marks: vec![Mark::Link {
                            href: "https://example.test/雪".into(),
                        }],
                    },
                    Inline::HardBreak,
                    Inline::Run {
                        text: "After break".into(),
                        marks: vec![],
                    },
                ],
            },
        }],
    });
    let compiled = compile(&desired, &current, facts);
    let witness = super::compiled(&desired, facts, &compiled).unwrap();
    assert!(witness.no_offload_at_witness);
    assert_eq!(witness.provisional_page_ids, 1);
    assert_eq!(witness.active_text_segments, 4);
    assert!(witness.scope.contains("conditional-on-save"));
    // This read-only calculation consumed real full cumulative updates, not
    // write(None) substitution. GC may compress deleted text, but its causal
    // clocks remain represented.
    let original_text = facts.iter().find(|row| row["attribute"] == "block/text").unwrap()["data"]["value"]
        .as_str()
        .unwrap();
    let new_text = compiled
        .mutations
        .iter()
        .find_map(|mutation| match mutation {
            NativeMutation::AssertFact(row)
                if row.attribute == "block/text" && row.entity.to_string() == current.state.nodes[&current.pages[0].blocks[0].key].entity =>
            {
                row.data["value"].as_str()
            }
            _ => None,
        })
        .unwrap();
    {
        use base64::Engine;
        use yrs::{ReadTxn, Transact, updates::decoder::Decode};
        let load = |payload: &str| {
            let doc = yrs::Doc::new();
            doc.transact_mut()
                .apply_update(yrs::Update::decode_v1(&base64::engine::general_purpose::STANDARD.decode(payload).unwrap()).unwrap())
                .unwrap();
            doc.transact().state_vector()
        };
        let old = load(original_text);
        let new = load(new_text);
        for (client, clock) in old.iter() {
            assert!(new.get(client) >= *clock);
        }
    }
    let same = super::compiled(&desired, facts, &compile(&desired, &current, facts)).unwrap();
    assert_eq!(
        same.pages_json_utf8_upper_bound, witness.pages_json_utf8_upper_bound,
        "different provisional UUIDs have the same known 36-byte width"
    );
}

#[test]
fn inactive_text_is_not_redecoded_or_counted_on_code_transition() {
    let evidence = fixtures();
    let fixture = &evidence["fixtures"][0];
    let current = document(fixture);
    let facts = fixture["facts"].as_array().unwrap();
    let mut desired = current.clone();
    desired.pages[0].blocks[0].kind = BlockKind::Code {
        language: "rust".into(),
        text: "fn main() {}".into(),
    };
    let compiled = compile(&desired, &current, facts);
    assert!(
        !compiled
            .mutations
            .iter()
            .any(|mutation| matches!(mutation, NativeMutation::AssertFact(row) if row.attribute == "block/text"))
    );
    let witness = super::compiled(&desired, facts, &compiled).unwrap();
    assert_eq!(witness.active_text_segments, 0);
    assert!(witness.no_offload_at_witness);
    assert!(witness.pages_json_utf8_upper_bound < 500);
}

#[test]
fn size_profile_refuses_nontext_list_wrapping_and_stale_compiler_text() {
    let evidence = fixtures();
    let fixture = &evidence["fixtures"][2];
    let mut facts = fixture["facts"].as_array().unwrap().clone();
    let current = document(fixture);
    let entity = current.state.nodes[&current.pages[0].blocks[0].key].entity.clone();
    facts.push(json!({"id":Uuid::new_v4(),"entity":entity,"attribute":"block/is-list","data":{"type":"boolean","value":true}}));
    // Current editor meaning doesn't model a native list around a code block;
    // never size the plain code wrapper and call that guaranteed inline.
    let compiled = compile(&current, &current, &facts);
    assert!(super::compiled(&current, &facts, &compiled).is_err());
    let fixture = &evidence["fixtures"][0];
    let current = document(fixture);
    let facts = fixture["facts"].as_array().unwrap();
    let compiled = compile(&current, &current, facts);
    let mut changed = current.clone();
    changed.pages[0].blocks[0].kind = BlockKind::Paragraph { content: vec![] };
    assert!(super::compiled(&changed, facts, &compiled).is_err());
}
