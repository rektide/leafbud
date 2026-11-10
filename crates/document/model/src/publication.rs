use super::*;

/// Compact publication progress. Editor achievement is checkpointed atomically
/// with the save receipt. Dispatch is marked BEFORE polling the effect future.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicationAttempt {
    pub id: Uuid,
    pub input_hash: String,
    pub editor_hash: String,
    pub stage: PublicationStage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "camelCase", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum PublicationStage {
    EditorAchieved,
    /// May have been sent, including a crash before the actual send. Never retry.
    Dispatched {
        owner: String,
        arguments_hash: String,
        core_hash: String,
        contributor: Option<String>,
        old_uri: Option<String>,
        old_cid: Option<String>,
        date: PublicationDate,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase", deny_unknown_fields)]
pub enum PublicationDate {
    Exact(String),
    /// Only a first missing date or an explicitly future date. No body wildcard.
    NativeNow {
        not_before: String,
        not_after: Option<String>,
    },
}
