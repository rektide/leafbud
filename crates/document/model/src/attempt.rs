use super::*;

const MAX_ID: u64 = (1 << 53) - 1;
const MAX_TARGETS: usize = 32_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WriterState {
    pub client_group_id: Uuid,
    pub client_id: Uuid,
    /// IDs below this exclusive high-water mark are never allocated again,
    /// even when prepared-but-unsent work is deliberately abandoned.
    pub next_mutation_id: u64,
}

impl WriterState {
    pub fn fresh() -> Self {
        Self {
            client_group_id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            next_mutation_id: 1,
        }
    }
    pub fn reserve(&mut self, count: usize) -> Result<MutationRange, &'static str> {
        if count == 0 || count > MAX_TARGETS {
            return Err("invalid mutation reservation size");
        }
        let next = self
            .next_mutation_id
            .checked_add(count as u64)
            .filter(|v| *v <= MAX_ID)
            .ok_or("mutation sequence exhausted")?;
        let range = MutationRange {
            first: self.next_mutation_id,
            last: next - 1,
        };
        self.next_mutation_id = next;
        Ok(range)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MutationRange {
    pub first: u64,
    pub last: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Attempt {
    pub id: Uuid,
    /// Prepared source's authored projection (tracking is excluded).
    pub input_hash: String,
    /// Desired keyed editor projection before effects, not a new receipt.
    pub desired_hash: String,
    pub effect: Effect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum Effect {
    Mint {
        base_url: String,
        /// Resolved intent: explicit null standalone, URI publication. This is
        /// required even before the token/complete binding can be learned.
        publication: MintContext,
        stage: MintStage,
    },
    Push {
        range: MutationRange,
        stage: PushStage,
        /// Proposed associations carry actual allocated IDs, not success.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        targets: BTreeMap<String, Node>,
        /// Explicit roots include detached entities which active pull can hide.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        created_entities: Vec<Uuid>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "uri", rename_all = "camelCase", deny_unknown_fields)]
pub enum MintContext {
    Standalone,
    Publication(String),
}
impl MintContext {
    pub fn publication(&self) -> Option<&str> {
        match self {
            Self::Standalone => None,
            Self::Publication(uri) => Some(uri),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "camelCase", deny_unknown_fields)]
pub enum MintStage {
    Prepared,
    /// Persisted BEFORE calling the mint. May have been sent; after restart
    /// absence of a response is uncertain, including the pre-send crash window.
    Dispatched,
    Located {
        token: Uuid,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "camelCase", deny_unknown_fields)]
pub enum PushStage {
    Prepared,
    /// Highest mutation in a request which may have left this process. Later
    /// reserved IDs are definitely unsent, not safe to independently re-encode.
    Dispatched {
        through: u64,
    },
    /// Acked terminal observation did not match the desired document. Retain
    /// identifiers for the next deliberate decision; never call it clean.
    Partial {
        through: u64,
    },
}

pub fn valid_fingerprint(hash: &str) -> bool {
    hash.strip_prefix("sha256:")
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

impl State {
    /// Lifecycle validation shared by parser and effect entrypoints. Pending
    /// locator without a complete binding and bound-without-receipt are valid.
    pub fn validate_attempt(&self) -> Result<(), &'static str> {
        if let Some(writer) = &self.writer {
            if writer.next_mutation_id == 0 || writer.next_mutation_id > MAX_ID || self.binding.is_none() {
                return Err("writer requires binding and a safe positive sequence");
            }
        }
        let Some(attempt) = &self.pending else {
            return Ok(());
        };
        if !valid_fingerprint(&attempt.input_hash) || !valid_fingerprint(&attempt.desired_hash) {
            return Err("invalid pending fingerprints");
        }
        match &attempt.effect {
            Effect::Mint { base_url, publication, stage } => {
                if base_url.is_empty() || publication.publication().is_some_and(str::is_empty) || self.sync.is_some() || self.writer.is_some() {
                    return Err("incoherent mint attempt");
                }
                if let Some(binding) = &self.binding {
                    if !matches!(stage, MintStage::Located { token } if *token == binding.token)
                        || binding.base_url != *base_url
                        || binding.publication.as_deref() != publication.publication()
                    {
                        return Err("mint binding differs from pending locator/context");
                    }
                }
            }
            Effect::Push {
                range,
                stage,
                targets,
                created_entities,
            } => {
                let writer = self.writer.as_ref().ok_or("push requires a writer")?;
                if range.first == 0
                    || range.last < range.first
                    || range.last >= writer.next_mutation_id
                    || range.last - range.first >= MAX_TARGETS as u64
                    || targets.len() > MAX_TARGETS
                    || created_entities.len() > MAX_TARGETS
                {
                    return Err("invalid pending mutation range/targets");
                }
                if let PushStage::Dispatched { through } | PushStage::Partial { through } = stage {
                    if *through < range.first || *through > range.last {
                        return Err("invalid dispatched high-water mark");
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> State {
        serde_json::from_value(serde_json::json!({"id":Uuid::new_v4(),"state":"draft"})).unwrap()
    }
    #[test]
    fn locator_precedes_binding_and_never_claims_agreement() {
        let mut state = state();
        state.pending = Some(Attempt {
            id: Uuid::new_v4(),
            input_hash: format!("sha256:{}", "a".repeat(64)),
            desired_hash: format!("sha256:{}", "b".repeat(64)),
            effect: Effect::Mint {
                base_url: "https://leaflet.pub".into(),
                publication: MintContext::Standalone,
                stage: MintStage::Located { token: Uuid::new_v4() },
            },
        });
        assert!(state.validate_attempt().is_ok());
        let round: State = serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();
        assert_eq!(round, state);
        state.sync = Some(Sync {
            profile: EDITOR_PROFILE.into(),
            local_hash: String::new(),
            remote_hash: String::new(),
        });
        assert!(state.validate_attempt().is_err());
        assert!(serde_json::from_value::<Attempt>(serde_json::json!({"id":Uuid::new_v4(),"inputHash":"x","desiredHash":"y","effect":{"kind":"mint","baseUrl":"https://leaflet.pub","stage":{"phase":"dispatched"}}})).is_err());
    }
    #[test]
    fn reserved_mutation_high_water_mark_is_monotonic_and_safe() {
        let mut writer = WriterState::fresh();
        assert_eq!(writer.reserve(250).unwrap(), MutationRange { first: 1, last: 250 });
        assert_eq!(writer.reserve(1).unwrap(), MutationRange { first: 251, last: 251 });
        assert!(writer.reserve(0).is_err());
        writer.next_mutation_id = MAX_ID;
        assert!(writer.reserve(1).is_err());
    }
}
