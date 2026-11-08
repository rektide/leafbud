use satteri_authoring_experiment::{
    author,
    facts::{self, BuildContext, Data},
    model::*,
    simulator::Store,
    text,
};

const GARDEN: &str = include_str!("../fixtures/garden.mdx");
fn plan(source: &str, generation: u128) -> facts::Plan {
    facts::build(&author::compile(source).unwrap(), BuildContext::fixture(generation)).unwrap()
}
fn run(text: &str) -> Vec<Inline> {
    vec![Inline::Run {
        text: text.into(),
        marks: vec![],
    }]
}

#[test]
fn real_fact_shapes_order_references_and_fresh_text_read_back() {
    let p = plan(GARDEN, 100);
    assert_eq!(p.pages.len(), 2);
    assert_eq!(p.polls.len(), 1);
    assert!(
        p.facts
            .iter()
            .any(|f| f.attribute == "page/type" && f.data == Data::PageType { value: "doc".into() })
    );
    let children: Vec<_> = p.facts.iter().filter(|f| f.entity == p.main_page && f.attribute == "card/block").collect();
    assert_eq!(children.len(), 10, "two lists lower to their ordered item blocks, no fake list entity");
    for (i, fact) in children.iter().enumerate() {
        assert_eq!(
            fact.data,
            Data::OrderedReference {
                value: fact.data.reference().unwrap().into(),
                position: format!("a{i}")
            }
        );
    }
    assert!(
        p.facts
            .iter()
            .any(|f| f.attribute == "block/card" && f.data.reference() == Some(p.pages["details"].as_str()))
    );
    for (entity, wanted) in &p.expected_text {
        let Data::Text { value } = &p.facts.iter().find(|f| &f.entity == entity && f.attribute == "block/text").unwrap().data else {
            panic!()
        };
        assert_eq!(&text::read(value).unwrap(), wanted);
    }
    let numbers: Vec<_> = p.facts.iter().filter(|f| f.attribute == "block/list-number").map(|f| &f.data).collect();
    assert_eq!(numbers, [&Data::Number { value: 3 }, &Data::Number { value: 4 }]);
    assert!(
        p.staged_mutations()
            .iter()
            .all(|m| ["createEntity", "assertFact"].contains(&m["name"].as_str().unwrap()))
    );
}

#[test]
fn fresh_unicode_marks_breaks_exact_newlines_and_empty_paragraph_work() {
    let mut content = vec![];
    push_run(&mut content, "é😀e\u{301}", &[Mark::Strong, Mark::Em]);
    content.push(Inline::HardBreak);
    push_run(
        &mut content,
        "  raw\ntext\t  ",
        &[Mark::Link {
            href: "https://example.test".into(),
        }],
    );
    assert_eq!(text::read(&text::fresh(&content, 901).unwrap()).unwrap(), content);
    assert_eq!(text::read(&text::fresh(&[], 902).unwrap()).unwrap(), vec![]);
    assert!(text::read("not-base64").is_err());
    assert!(text::fresh(&content, 1 << 53).is_err());
}

#[test]
fn initial_update_replay_is_safe_but_fresh_independent_text_does_not_replace_old() {
    let old = text::fresh(&run("old"), 8001).unwrap();
    assert_eq!(text::read(&text::merge(&old, &old).unwrap()).unwrap(), run("old"));
    let new = text::fresh(&run("new"), 8002).unwrap();
    assert!(
        text::read(&text::merge(&old, &new).unwrap()).is_err(),
        "two paragraphs, not replacement; fresh entity generation is required"
    );
}

#[test]
fn stage_is_detached_then_verified_explicit_entry_switch_keeps_binding_and_navigation() {
    let p = plan(GARDEN, 100);
    let mut store = Store::fixture();
    let binding = store.binding.clone();
    let old = store.active_page().to_owned();
    let navigation = store
        .facts
        .values()
        .find(|f| f.attribute == "root/page" && f.id != binding.entry_fact)
        .unwrap()
        .clone();
    store.stage(&p, None).unwrap();
    store.verify(&p).unwrap();
    assert_eq!(store.active_page(), old);
    assert!(!store.reachable(&binding.root).contains(&p.main_page));
    let switch = store.switch(&p).unwrap();
    assert_eq!(switch["name"], "assertFact");
    assert_eq!(switch["args"]["id"], binding.entry_fact);
    assert_eq!(switch["args"]["data"]["position"], "a0");
    assert_eq!(store.active_page(), p.main_page);
    assert_eq!(store.binding, binding);
    assert_eq!(store.facts[&navigation.id], navigation);
    let reachable = store.reachable(store.active_page());
    assert!(p.entities.is_subset(&reachable), "fixture definitions all reached through block/card");
    assert!(!reachable.contains(&old));
    assert!(store.entities.contains(&old), "no automatic cleanup");
}

#[test]
fn failed_or_corrupt_staging_never_exposes_unverified_body() {
    let p = plan(GARDEN, 100);
    let mut store = Store::fixture();
    let old = store.active_page().to_owned();
    assert!(store.stage(&p, Some(5)).is_err());
    assert_eq!(store.active_page(), old);
    assert!(store.switch(&p).is_err());
    assert_eq!(store.active_page(), old);
    store.stage(&p, None).unwrap();
    let text_id = store.facts.values().find(|f| f.attribute == "block/text").unwrap().id.clone();
    store.facts.get_mut(&text_id).unwrap().data = Data::Text {
        value: text::fresh(&run("corrupt"), 9999).unwrap(),
    };
    assert!(store.switch(&p).is_err());
    assert_eq!(store.active_page(), old);
}

#[test]
fn every_interrupted_fact_boundary_can_resume_without_reasserting_text() {
    let p = plan(GARDEN, 100);
    for boundary in 0..p.facts.len() {
        let mut store = Store::fixture();
        let old = store.active_page().to_owned();
        assert!(store.stage(&p, Some(boundary)).is_err());
        assert_eq!(store.active_page(), old);
        store.apply(&p).unwrap_or_else(|e| panic!("boundary {boundary}: {e}"));
        store.verify(&p).unwrap();
        assert_eq!(store.active_page(), p.main_page);
    }
}

#[test]
fn verifier_rejects_extra_memberships_and_duplicate_profile_facts() {
    let p = plan(GARDEN, 100);
    for attribute in ["card/block", "page/type"] {
        let mut store = Store::fixture();
        let old = store.active_page().to_owned();
        store.stage(&p, None).unwrap();
        let mut extra = p.facts.iter().find(|f| f.entity == p.main_page && f.attribute == attribute).unwrap().clone();
        extra.id = uuid::Uuid::from_u128(99999).to_string();
        store.facts.insert(extra.id.clone(), extra);
        assert!(store.switch(&p).is_err());
        assert_eq!(store.active_page(), old);
    }
}

#[test]
fn exact_repeat_lost_switch_response_and_semantically_equal_reencoding_do_not_append() {
    let p = plan(GARDEN, 100);
    let mut store = Store::fixture();
    store.apply(&p).unwrap();
    let count = store.facts.len();
    let before = serde_json::to_value(&store).unwrap();
    store.apply(&p).unwrap();
    store.switch(&p).unwrap();
    assert_eq!(store.facts.len(), count);
    assert_eq!(serde_json::to_value(&store).unwrap(), before);
    let doc = author::compile(GARDEN).unwrap();
    let mut context = BuildContext::fixture(100);
    context.first_text_client = 9001;
    let reencoded = facts::build(&doc, context).unwrap();
    assert!(
        p.facts
            .iter()
            .zip(&reencoded.facts)
            .any(|(a, b)| matches!(a.data, Data::Text { .. }) && a.data != b.data)
    );
    store.apply(&reencoded).unwrap();
    assert_eq!(
        serde_json::to_value(&store).unwrap(),
        before,
        "existing equal text is left untouched, never merged with independent text"
    );
}

#[test]
fn source_edit_gets_fresh_body_and_prose_ids_but_retains_author_keys_and_poll() {
    let first = plan(GARDEN, 100);
    let edited = GARDEN.replace("A patient garden", "A careful garden");
    let next = plan(&edited, 101);
    let mut store = Store::fixture();
    store.apply(&first).unwrap();
    let binding = store.binding.clone();
    store.apply(&next).unwrap();
    assert_eq!(store.binding, binding);
    assert_ne!(first.main_page, next.main_page);
    assert_ne!(first.author_keys["introduction"].entity, next.author_keys["introduction"].entity);
    assert_eq!(first.author_keys.keys().collect::<Vec<_>>(), next.author_keys.keys().collect::<Vec<_>>());
    assert_eq!(first.polls, next.polls, "stable named Poll reuses its configuration and entity");
    let reachable = store.reachable(store.active_page());
    assert!(!reachable.contains(&first.main_page));
    assert!(reachable.contains(&next.main_page));
    assert_eq!(store.facts.values().filter(|f| f.attribute == "root/page").count(), 2);
}

#[test]
fn reordering_keyed_prose_preserves_author_name_not_array_path_or_crdt_identity() {
    let source = "---\nformat: springleaf-mdx/1\ntitle: T\n---\n\n<Key id=\"a\" />\nAlpha.\n\n<Key id=\"b\" />\nBeta.\n";
    let moved = source.replace(
        "<Key id=\"a\" />\nAlpha.\n\n<Key id=\"b\" />\nBeta.",
        "<Key id=\"b\" />\nBeta.\n\n<Key id=\"a\" />\nAlpha.",
    );
    let a = plan(source, 100);
    let b = plan(&moved, 101);
    assert_eq!(a.author_keys.keys().collect::<Vec<_>>(), b.author_keys.keys().collect::<Vec<_>>());
    assert_ne!(a.author_keys["a"].source, b.author_keys["a"].source);
    assert_ne!(a.author_keys["a"].entity, b.author_keys["a"].entity);
}

#[test]
fn changing_or_removing_stable_poll_options_requires_new_key_before_any_writes() {
    let first = plan(GARDEN, 100);
    let mut store = Store::fixture();
    store.apply(&first).unwrap();
    let before = serde_json::to_value(&store).unwrap();
    for edit in [
        GARDEN.replace("Gardens & maps", "Different meaning"),
        GARDEN.replace(", \"  exact spacing  \"", ""),
    ] {
        let next = plan(&edit, 101);
        assert!(store.apply(&next).is_err());
        assert_eq!(serde_json::to_value(&store).unwrap(), before);
    }
    let changed = GARDEN
        .replace("Gardens & maps", "Different meaning")
        .replace("key=\"next-topic\"", "key=\"new-topic\"");
    store.apply(&plan(&changed, 102)).unwrap();
    assert!(store.reachable(store.active_page()).is_disjoint(&first.polls.keys().cloned().collect()));
}
