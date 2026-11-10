//! Synthetic native store, based on Leaflet mutations.ts / cachedServerMutationContext
//! at 600c7b5f. Independent of the authoring compiler; HTTP callers supply only
//! native wire requests. Deliberately not a deployment/authorization simulator.
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use yrs::{Doc, ReadTxn, StateVector, Transact, Update, updates::decoder::Decode};

pub const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
pub const ROOT: &str = "11111111-1111-4111-8111-111111111111";
pub const PAGE: &str = "22222222-2222-4222-8222-222222222222";
pub const SEED: &str = "33333333-3333-4333-8333-333333333333";
pub const SET: &str = "44444444-4444-4444-8444-444444444444";
pub const ENTRY: &str = "55555555-5555-4555-8555-555555555555";
pub const MEMBER: &str = "66666666-6666-4666-8666-666666666666";
pub const URI: &str = "at://did:plc:synthetic/site.standard.publication/garden";

pub struct Store {
    pub data: Value,
    pub facts: BTreeMap<String, Value>,
    pub extras: BTreeMap<String, Value>,
    pub entities: BTreeSet<String>,
    pub acks: BTreeMap<String, BTreeMap<String, u64>>,
    pub pushes: usize,
    pub skip_attachments: bool,
    pub skip_all: bool,
    pub no_ack: bool,
    clock: usize,
}
impl Store {
    pub fn new(publication: bool) -> Self {
        let mut store = Self {
            data: json!({"id":TOKEN,"root_entity":ROOT,"title":null,"description":null,
                "permission_token_rights":[{"token":TOKEN,"entity_set":SET,"read":true,"write":true}],
                "leaflets_in_publications":if publication{json!([{"publication":URI,"doc":null,"title":"Untitled"}])}else{json!([])},"leaflets_to_documents":[]}),
            facts: BTreeMap::new(),
            extras: BTreeMap::from([("initialized".into(), json!(true))]),
            entities: [ROOT, PAGE, SEED].into_iter().map(str::to_owned).collect(),
            acks: BTreeMap::new(),
            pushes: 0,
            skip_attachments: false,
            skip_all: false,
            no_ack: false,
            clock: 0,
        };
        for row in [
            json!({"id":ENTRY,"entity":ROOT,"attribute":"root/page","data":{"type":"ordered-reference","value":PAGE,"position":"a0"}}),
            json!({"id":MEMBER,"entity":PAGE,"attribute":"card/block","data":{"type":"ordered-reference","value":SEED,"position":"a0"}}),
            json!({"id":"77777777-7777-4777-8777-777777777777","entity":SEED,"attribute":"block/type","data":{"type":"block-type-union","value":if publication{"text"}else{"heading"}}}),
            json!({"id":"99999999-9999-4999-8999-999999999999","entity":ROOT,"attribute":"theme/page-leaflet-watermark","data":{"type":"boolean","value":true}}),
        ] {
            store.facts.insert(row["id"].as_str().unwrap().into(), row);
        }
        if !publication {
            store.facts.insert(
                "88888888-8888-4888-8888-888888888888".into(),
                json!({"id":"88888888-8888-4888-8888-888888888888","entity":SEED,"attribute":"block/heading-level","data":{"type":"number","value":1}}),
            );
        } else {
            store.extras.insert("draft_contributors".into(), json!(["did:plc:contributor-default"]));
        }
        store
    }
    pub fn rows(&self, root: &str, depth: usize) -> Vec<Value> {
        let mut entities = BTreeSet::from([root.to_owned()]);
        let mut pending = entities.clone();
        for _ in 0..depth {
            let mut next = BTreeSet::new();
            for row in self.facts.values().filter(|r| pending.contains(r["entity"].as_str().unwrap())) {
                if matches!(row["data"]["type"].as_str(), Some("reference" | "ordered-reference" | "spatial-reference")) {
                    let target = row["data"]["value"].as_str().unwrap().to_owned();
                    if entities.insert(target.clone()) {
                        next.insert(target);
                    }
                }
            }
            pending = next;
        }
        self.facts
            .values()
            .filter(|r| entities.contains(r["entity"].as_str().unwrap()))
            .cloned()
            .collect()
    }
    pub fn one(&self, entity: &str, attr: &str) -> Option<&Value> {
        self.facts.values().find(|r| r["entity"] == entity && r["attribute"] == attr)
    }
    pub fn assert(&mut self, mut row: Value) {
        let many = matches!(row["attribute"].as_str().unwrap(), "card/block" | "root/page");
        if !many && let Some(old) = self.one(row["entity"].as_str().unwrap(), row["attribute"].as_str().unwrap()) {
            row["id"] = old["id"].clone();
            if row["attribute"] == "block/text" {
                // Genuine Yrs merge, not assignment of a compiler's expected projection.
                let doc = Doc::new();
                doc.transact_mut()
                    .apply_update(Update::decode_v1(&STANDARD.decode(old["data"]["value"].as_str().unwrap()).unwrap()).unwrap())
                    .unwrap();
                doc.transact_mut()
                    .apply_update(Update::decode_v1(&STANDARD.decode(row["data"]["value"].as_str().unwrap()).unwrap()).unwrap())
                    .unwrap();
                row["data"]["value"] = STANDARD.encode(doc.transact().encode_state_as_update_v1(&StateVector::default())).into();
            }
        }
        assert!(self.entities.contains(row["entity"].as_str().unwrap()), "unknown entity in native assertion");
        self.facts.insert(row["id"].as_str().unwrap().into(), row);
    }
    fn end(&mut self) -> String {
        self.clock += 1;
        format!("z{:08}", self.clock)
    }
    pub fn rpc(&mut self, path: &str, body: &Value) -> Option<Value> {
        Some(match path {
            "/api/rpc/get_leaflet_data" => json!({"result":{"error":null,"data":self.data}}),
            "/api/rpc/pull" => {
                let mut patch = vec![json!({"op":"clear"})];
                patch.extend(self.extras.iter().map(|(k, v)| json!({"op":"put","key":k,"value":v})));
                patch.extend(self.rows(ROOT, 64).into_iter().map(|r| json!({"op":"put","key":r["id"],"value":r})));
                json!({"patch":patch,"lastMutationIDChanges":self.acks.get(body["pullRequest"]["clientGroupID"].as_str().unwrap()).cloned().unwrap_or_default()})
            }
            "/api/rpc/getFactsForRoots" => {
                let facts: BTreeMap<_, _> = body["roots"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| {
                        let root = r.as_str().unwrap();
                        let rows: Vec<_> = self
                            .rows(root, 3)
                            .into_iter()
                            .map(|mut r| {
                                r["root_id"] = root.into();
                                r
                            })
                            .collect();
                        (root.to_owned(), rows)
                    })
                    .collect();
                json!({"result":{"facts":facts}})
            }
            "/api/rpc/push" => {
                self.pushes += 1;
                let group = body["pushRequest"]["clientGroupID"].as_str().unwrap();
                for mutation in body["pushRequest"]["mutations"].as_array().unwrap() {
                    let client = mutation["clientID"].as_str().unwrap();
                    let id = mutation["id"].as_u64().unwrap();
                    if self.acks.get(group).and_then(|a| a.get(client)).is_some_and(|ack| *ack >= id) {
                        continue;
                    }
                    let args = &mutation["args"];
                    if !self.skip_all {
                        match mutation["name"].as_str().unwrap() {
                            "createEntity" => {
                                for r in args.as_array().unwrap() {
                                    assert_eq!(r["permission_set"], SET);
                                    self.entities.insert(r["entityID"].as_str().unwrap().into());
                                }
                            }
                            "assertFact" => self.assert(args.clone()),
                            "retractFact" => {
                                self.facts.remove(args["factID"].as_str().unwrap());
                            }
                            "addLastBlock" if !self.skip_attachments => {
                                let position = self.end();
                                self.assert(json!({"id":args["factID"],"entity":args["parent"],"attribute":"card/block","data":{"type":"ordered-reference","value":args["entity"],"position":position}}));
                            }
                            "moveBlock" if !self.skip_attachments => {
                                assert_eq!(args["position"]["type"], "end");
                                let row = self
                                    .facts
                                    .values()
                                    .find(|r| r["entity"] == args["oldParent"] && r["attribute"] == "card/block" && r["data"]["value"] == args["block"])
                                    .cloned();
                                if let Some(mut row) = row {
                                    row["entity"] = args["newParent"].clone();
                                    row["data"]["position"] = self.end().into();
                                    self.assert(row);
                                }
                            }
                            "addLastBlock" | "moveBlock" => {}
                            "updateLeafletMetadata" => {
                                for field in ["title", "description"] {
                                    if let Some(value) = args.get(field) {
                                        self.data[field] = value.clone();
                                    }
                                }
                            }
                            "updatePublicationDraft" => {
                                for field in ["title", "description", "tags"] {
                                    if let Some(value) = args.get(field) {
                                        self.extras.insert(format!("publication_{field}"), value.clone());
                                    }
                                }
                                if let Some(value) = args.get("preferences") {
                                    self.extras.insert("post_preferences".into(), value.clone());
                                    let table = if self.data["leaflets_in_publications"].as_array().is_some_and(|v| !v.is_empty()) {
                                        "leaflets_in_publications"
                                    } else {
                                        "leaflets_to_documents"
                                    };
                                    self.data[table][0]["preferences"] = value.clone();
                                }
                            }
                            name => panic!("unexpected mutation {name}"),
                        }
                    }
                    if !self.no_ack {
                        self.acks.entry(group.into()).or_default().insert(client.into(), id);
                    }
                }
                json!({})
            }
            _ => return None,
        })
    }
}
