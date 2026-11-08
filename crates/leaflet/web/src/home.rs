//! RSC extraction recovered from pre-reset 80436b61, with raw rows retained,
//! strict known-shape validation, two lifecycle states and explicit coverage.
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Error, LeafletData, Metadata};

static PUSH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)self\.__next_f\.push\((\[.*?\])\)</script>").expect("push regex"));
static HREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"href="/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})""#).expect("href regex"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub did: Option<String>,
    pub handle: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Discovery {
    pub identity: Identity,
    pub fetched_at: Option<i64>,
    pub href_payload_mismatch: Vec<String>,
    pub leaflets: Vec<DiscoveredLeaflet>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredLeaflet {
    pub metadata: Metadata,
    pub created_at: Option<String>,
    pub created_at_source: Option<&'static str>,
    pub archived: bool,
    pub on_homepage: bool,
    pub contributor: bool,
    /// Exact source rows, excluding unrelated account/identity data.
    pub raw_home_rows: Vec<Value>,
}

#[derive(Deserialize)]
struct HomeIdentity {
    atp_did: Option<String>,
    bsky_profiles: Option<Identity>,
    fetched_at: Option<i64>,
    permission_token_on_homepage: Vec<Value>,
    contributor_leaflets: Vec<Value>,
}

pub fn parse_home(html: &str) -> Result<Discovery, Error> {
    let stream = flight_stream(html)?;
    let slice = enclosing_object(&stream, "permission_token_on_homepage").ok_or(Error::NoHomePayload)?;
    let identity: HomeIdentity = serde_json::from_str(slice).map_err(|_| Error::Home("invalid identity/row arrays"))?;
    let profile = identity.bsky_profiles;
    let mut leaflets: Vec<DiscoveredLeaflet> = Vec::new();
    let mut index = HashMap::new();
    for (rows, contributor) in [(identity.permission_token_on_homepage, false), (identity.contributor_leaflets, true)] {
        for row in rows {
            let raw = row
                .get("permission_tokens")
                .filter(|v| v.is_object())
                .ok_or(Error::Home("missing permission_tokens object"))?
                .clone();
            let data = LeafletData::from_raw(raw).map_err(|_| Error::Home("invalid token metadata/rights/linkage"))?;
            let id = data.metadata.id;
            if let Some(&i) = index.get(&id) {
                let item: &mut DiscoveredLeaflet = &mut leaflets[i];
                item.contributor |= contributor;
                item.on_homepage |= !contributor;
                item.raw_home_rows.push(row);
                continue;
            }
            let created_at = match row.get("created_at") {
                None | Some(Value::Null) => None,
                Some(Value::String(s)) => Some(s.clone()),
                _ => return Err(Error::Home("invalid created_at")),
            };
            let from_row = created_at.is_some();
            let created_at = created_at.or_else(|| {
                (id.get_version_num() == 7)
                    .then(|| (id.as_u128() >> 80) as i64)
                    .and_then(|ms| jiff::Timestamp::from_millisecond(ms).ok())
                    .map(|t| t.to_string())
            });
            let created_at_source = if from_row { Some("row") } else { created_at.as_ref().map(|_| "uuidv7") };
            index.insert(id, leaflets.len());
            leaflets.push(DiscoveredLeaflet {
                metadata: data.metadata,
                created_at,
                created_at_source,
                archived: row.get("archived").is_some_and(|v| !v.is_null()),
                on_homepage: !contributor,
                contributor,
                raw_home_rows: vec![row],
            });
        }
    }
    // Compare actual instants rather than differently offset timestamp strings.
    leaflets.sort_by_key(|l| std::cmp::Reverse((l.created_at.as_ref().and_then(|s| s.parse::<jiff::Timestamp>().ok()), l.metadata.id)));
    let home_ids: HashSet<_> = leaflets.iter().filter(|l| l.on_homepage).map(|l| l.metadata.id.to_string()).collect();
    let href_ids: HashSet<_> = HREF_RE.captures_iter(html).map(|c| c[1].to_string()).collect();
    let mut href_payload_mismatch: Vec<_> = home_ids.symmetric_difference(&href_ids).cloned().collect();
    href_payload_mismatch.sort();
    Ok(Discovery {
        identity: Identity {
            did: profile.as_ref().and_then(|p| p.did.clone()).or(identity.atp_did),
            handle: profile.and_then(|p| p.handle),
        },
        fetched_at: identity.fetched_at,
        href_payload_mismatch,
        leaflets,
    })
}

pub(crate) fn flight_stream(html: &str) -> Result<String, Error> {
    let mut out = String::new();
    for caps in PUSH_RE.captures_iter(html) {
        let arr: Value = serde_json::from_str(&caps[1]).map_err(|_| Error::Home("invalid flight chunk JSON"))?;
        // Next also emits bootstrap/control arrays such as [0] and [2,null].
        if arr.get(0).and_then(Value::as_u64) != Some(1) {
            continue;
        }
        out.push_str(arr.get(1).and_then(Value::as_str).ok_or(Error::Home("non-string flight chunk"))?);
    }
    Ok(out)
}

/// Brace matcher from the proven extractor; skip escaped strings so prose
/// containing braces cannot change the enclosing identity object.
pub(crate) fn enclosing_object<'a>(stream: &'a str, key: &str) -> Option<&'a str> {
    let from = stream.find(&format!("\"{key}\":"))?;
    let bytes = stream.as_bytes();
    let mut depth = 0_i64;
    let mut j = from;
    let start = loop {
        if j == 0 {
            return None;
        }
        j -= 1;
        match bytes[j] {
            b'"' => loop {
                if j == 0 {
                    break;
                }
                j -= 1;
                if bytes[j] == b'"' {
                    let mut k = j;
                    while k > 0 && bytes[k - 1] == b'\\' {
                        k -= 1;
                    }
                    if (j - k) % 2 == 0 {
                        break;
                    }
                }
            },
            b'}' | b']' => depth += 1,
            b'{' if depth == 0 => break j,
            b'{' | b'[' => depth -= 1,
            _ => {}
        }
    };
    let (mut depth, mut in_string, mut escaped) = (0, false, false);
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&stream[start..=i]);
                    }
                }
                _ => {}
            }
        }
    }
    None
}
