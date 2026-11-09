//! Null-cookie full snapshots for read-only followers and actual writer groups.
use crate::{Error, LeafletClient, transport};
use leafbud_protocol::{ExplicitFacts, Writer};
use serde_json::Value;
use uuid::Uuid;

pub type PullSnapshot = leafbud_protocol::FullView;

pub fn parse_pull(raw: Value) -> Result<PullSnapshot, Error> {
    PullSnapshot::parse(raw).map_err(|e| Error::Pull(e.0))
}

impl LeafletClient {
    pub async fn pull(&self, id: Uuid) -> Result<PullSnapshot, Error> {
        self.pull_group(id, "leafbud-read-only").await
    }

    /// Writer acknowledgments MUST use the actual group's null-cookie view;
    /// an absent/lower ack never proves an earlier request cannot land later.
    pub async fn pull_writer(&self, id: Uuid, writer: Writer) -> Result<PullSnapshot, Error> {
        self.pull_group(id, &writer.client_group_id.to_string()).await
    }

    async fn pull_group(&self, id: Uuid, group: &str) -> Result<PullSnapshot, Error> {
        let response = self
            .http
            .post(self.base.join("api/rpc/pull").expect("static path"))
            .json(&serde_json::json!({"token_id":id,"pullRequest":{
                "pullVersion":1,"schemaVersion":"","profileID":"leafbud-read-only",
                "cookie":null,"clientGroupID":group
            }}))
            .send()
            .await
            .map_err(transport)?;
        let raw = self.read_json(response, "POST pull").await?;
        parse_pull(raw)
    }

    /// Pinned native RPC: `{roots:[entity UUID]}` → `{result:{facts:{root:rows}}}`.
    /// Server traversal depth is fixed at 3. Roots may be detached entities;
    /// request known fact IDs by their known owning entity, then select exactly.
    /// Empty results do NOT prove entity nonexistence or token access rights.
    pub async fn facts_for_entities(&self, entities: &[Uuid]) -> Result<ExplicitFacts, Error> {
        if entities.is_empty() || entities.len() > 128 {
            return Err(Error::Native("explicit root count must be 1..128"));
        }
        let response = self
            .http
            .post(self.base.join("api/rpc/getFactsForRoots").expect("static path"))
            .json(&serde_json::json!({"roots":entities}))
            .send()
            .await
            .map_err(transport)?;
        ExplicitFacts::parse(self.read_json(response, "POST getFactsForRoots").await?, entities).map_err(|e| Error::Native(e.0))
    }
}
