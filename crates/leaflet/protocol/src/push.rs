use super::*;

/// Actual server mutation names/arguments; not a generic operation AST. The
/// compiler owns supported meaning/causality, the transport owns envelope IDs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "name", content = "args", rename_all = "camelCase", deny_unknown_fields)]
pub enum NativeMutation {
    CreateEntity(Vec<CreateEntity>),
    AssertFact(AssertFact),
    RetractFact {
        #[serde(rename = "factID")]
        fact_id: Uuid,
    },
    AddLastBlock {
        parent: Uuid,
        #[serde(rename = "factID")]
        fact_id: Uuid,
        entity: Uuid,
    },
    MoveBlock {
        #[serde(rename = "oldParent")]
        old_parent: Uuid,
        block: Uuid,
        #[serde(rename = "newParent")]
        new_parent: Uuid,
        position: Position,
    },
    UpdateLeafletMetadata(LeafletMetadata),
    UpdatePublicationDraft(PublicationMetadata),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateEntity {
    #[serde(rename = "entityID")]
    pub entity_id: Uuid,
    pub permission_set: Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssertFact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub entity: Uuid,
    pub attribute: String,
    /// Open native data boundary; validation below restricts executable shapes.
    pub data: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Position {
    First,
    End,
    Before { entity: Uuid },
    After { entity: Uuid },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeafletMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// None omits; Some(None) writes null. Custom decode retains that difference.
    #[serde(default, deserialize_with = "nullable", skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
}
fn nullable<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(d).map(Some)
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Explicit allowed native IDs from current facts and planned new targets.
/// Passing arbitrary request bytes cannot bypass this check or set token/group.
#[derive(Clone, Debug, Default)]
pub struct WriteScope {
    pub entities: BTreeSet<Uuid>,
    pub facts: BTreeSet<Uuid>,
    pub publication: bool,
}

impl NativeMutation {
    pub fn validate(&self, target: Target, scope: &WriteScope) -> Result<(), Error> {
        let entity = |id: &Uuid| {
            if scope.entities.contains(id) {
                Ok(())
            } else {
                Err(Error("entity outside validated scope"))
            }
        };
        let fact = |id: &Uuid| {
            if scope.facts.contains(id) {
                Ok(())
            } else {
                Err(Error("fact outside validated scope"))
            }
        };
        match self {
            Self::CreateEntity(rows) => {
                if rows.is_empty() || rows.len() > 250 {
                    return Err(Error("invalid createEntity array"));
                }
                for row in rows {
                    entity(&row.entity_id)?;
                    if row.permission_set != target.entity_set {
                        return Err(Error("entity set differs from validated target"));
                    }
                }
            }
            Self::AssertFact(row) => {
                entity(&row.entity)?;
                if let Some(id) = &row.id {
                    fact(id)?;
                }
                let data = row.data.as_object().ok_or(Error("invalid fact data"))?;
                let (kind, reference, many) = match row.attribute.as_str() {
                    "root/page" | "card/block" => ("ordered-reference", true, true),
                    "block/card" => ("reference", true, false),
                    "block/type" => ("block-type-union", false, false),
                    "page/type" => ("page-type-union", false, false),
                    "block/text" => ("text", false, false),
                    "block/heading-level" | "block/list-number" => ("number", false, false),
                    "block/is-list" | "theme/page-leaflet-watermark" => ("boolean", false, false),
                    "block/list-style" => ("list-style-union", false, false),
                    "block/code" | "block/code-language" | "image/alt" => ("string", false, false),
                    "block/image" => ("image", false, false),
                    _ => return Err(Error("unsupported executable fact attribute")),
                };
                if data.get("type").and_then(Value::as_str) != Some(kind) || (many && row.id.is_none()) {
                    return Err(Error("fact type/cardinality mismatch"));
                }
                let allowed: &[&str] = match kind {
                    "ordered-reference" => &["type", "value", "position"],
                    "image" => &["type", "src", "fallback", "width", "height"],
                    _ => &["type", "value"],
                };
                if data.keys().any(|k| !allowed.contains(&k.as_str())) {
                    return Err(Error("unknown executable data fields"));
                }
                if kind == "ordered-reference" && data["position"].as_str().is_none_or(str::is_empty) {
                    return Err(Error("missing placement position"));
                }
                if reference {
                    let id = data["value"].as_str().and_then(|s| Uuid::parse_str(s).ok()).ok_or(Error("invalid reference"))?;
                    entity(&id)?;
                } else if kind == "number" {
                    if !data["value"].is_number() {
                        return Err(Error("invalid number"));
                    }
                } else if kind == "boolean" {
                    if !data["value"].is_boolean() {
                        return Err(Error("invalid boolean"));
                    }
                } else if kind == "image" {
                    if data["src"].as_str().is_none_or(str::is_empty)
                        || ["fallback"].iter().any(|k| data.get(*k).is_some_and(|v| !v.is_null() && !v.is_string()))
                        || ["width", "height"]
                            .iter()
                            .any(|k| data.get(*k).is_some_and(|v| !v.is_null() && v.as_u64().is_none()))
                    {
                        return Err(Error("invalid image"));
                    }
                } else if !data["value"].is_string() {
                    return Err(Error("invalid string/union/text"));
                }
                let value = &data["value"];
                if row.attribute == "block/type"
                    && !matches!(
                        value.as_str(),
                        Some("text" | "heading" | "blockquote" | "code" | "horizontal-rule" | "image" | "card")
                    )
                    || row.attribute == "page/type" && value != "doc"
                    || row.attribute == "block/list-style" && !matches!(value.as_str(), Some("unordered" | "ordered"))
                    || row.attribute == "block/heading-level" && value.as_u64().is_none_or(|v| !(1..=6).contains(&v))
                {
                    return Err(Error("unsupported executable union/heading value"));
                }
            }
            Self::RetractFact { fact_id } => fact(fact_id)?,
            Self::AddLastBlock {
                parent,
                fact_id,
                entity: child,
            } => {
                entity(parent)?;
                entity(child)?;
                fact(fact_id)?;
            }
            Self::MoveBlock {
                old_parent,
                block,
                new_parent,
                position,
            } => {
                entity(old_parent)?;
                entity(block)?;
                entity(new_parent)?;
                if let Position::Before { entity: id } | Position::After { entity: id } = position {
                    entity(id)?;
                }
            }
            Self::UpdateLeafletMetadata(_) if scope.publication => return Err(Error("standalone metadata in publication context")),
            Self::UpdatePublicationDraft(_) if !scope.publication => return Err(Error("publication metadata in standalone context")),
            Self::UpdateLeafletMetadata(_) | Self::UpdatePublicationDraft(_) => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BatchLimits {
    pub mutations: usize,
    pub bytes: usize,
    pub requests: usize,
}
impl Default for BatchLimits {
    fn default() -> Self {
        Self {
            mutations: 250,
            bytes: 1024 * 1024,
            requests: 128,
        }
    }
}

/// Immutable exact encoded request, constructed only with the validated target.
/// Cloning/retrying this value never re-encodes CRDT or changes request identity.
#[derive(Clone)]
pub struct EncodedPush {
    target: Target,
    writer: Writer,
    first: u64,
    last: u64,
    bytes: Vec<u8>,
}
impl std::fmt::Debug for EncodedPush {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncodedPush")
            .field("range", &(self.first, self.last))
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
impl EncodedPush {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn target(&self) -> Target {
        self.target
    }
    pub fn writer(&self) -> Writer {
        self.writer
    }
    pub fn first(&self) -> u64 {
        self.first
    }
    pub fn last(&self) -> u64 {
        self.last
    }
}

/// Validate and encode ALL mutations before returning any request. Native
/// semantics/read-back are the editor's responsibility, not HTTP success.
pub fn encode_pushes(
    target: Target,
    writer: Writer,
    first: u64,
    timestamp: u64,
    mutations: &[NativeMutation],
    scope: &WriteScope,
    limits: BatchLimits,
) -> Result<Vec<EncodedPush>, Error> {
    if first == 0
        || first > MAX_SAFE_INTEGER
        || timestamp > MAX_SAFE_INTEGER
        || limits.mutations == 0
        || limits.mutations > 250
        || limits.bytes == 0
        || limits.requests == 0
    {
        return Err(Error("invalid request bounds/sequence"));
    }
    first
        .checked_add(mutations.len().saturating_sub(1) as u64)
        .filter(|id| *id <= MAX_SAFE_INTEGER)
        .ok_or(Error("mutation ID overflow"))?;
    let mut encoded = Vec::new();
    for (index, mutation) in mutations.iter().enumerate() {
        mutation.validate(target, scope)?;
        let mut value = serde_json::to_value(mutation).map_err(|_| Error("mutation serialization"))?;
        value["id"] = (first + index as u64).into();
        value["clientID"] = writer.client_id.to_string().into();
        value["timestamp"] = timestamp.into();
        encoded.push(value);
    }
    let encode = |rows: &[Value]| {
        serde_json::to_vec(&serde_json::json!({
            "rootEntity":target.root,"token":{"id":target.token},"pushRequest":{
                "pushVersion":1,"schemaVersion":"","profileID":"leafbud-writer","clientGroupID":writer.client_group_id,"mutations":rows
            }
        }))
        .map_err(|_| Error("request serialization"))
    };
    let mut result = Vec::new();
    let mut offset = 0;
    let overhead = encode(&[])?.len();
    let sizes: Vec<_> = encoded.iter().map(|row| serde_json::to_vec(row).expect("JSON value").len()).collect();
    while offset < encoded.len() {
        let mut end = offset;
        let mut size = overhead;
        while end < encoded.len() && end - offset < limits.mutations {
            let next = size.saturating_add(sizes[end]).saturating_add(usize::from(end > offset));
            if next > limits.bytes {
                break;
            }
            size = next;
            end += 1;
        }
        if end == offset {
            return Err(Error("single mutation exceeds byte limit"));
        }
        let bytes = encode(&encoded[offset..end])?;
        if result.len() == limits.requests {
            return Err(Error("request count exceeds limit"));
        }
        result.push(EncodedPush {
            target,
            writer,
            first: first + offset as u64,
            last: first + end as u64 - 1,
            bytes,
        });
        offset = end;
    }
    Ok(result)
}
