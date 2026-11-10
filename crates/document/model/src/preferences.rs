//! Four post visibility controls, not publication-level settings or delivery.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const PREFERENCE_KEYS: [&str; 4] = ["showInDiscover", "showComments", "showMentions", "showRecommends"];

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_in_discover: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_mentions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_recommends: Option<bool>,
}

impl<'de> Deserialize<'de> for Preferences {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let value = Value::deserialize(d)?;
        let map = value.as_object().ok_or_else(|| D::Error::custom("preferences must be a mapping"))?;
        let mut preferences = Self::default();
        for (key, value) in map {
            let value = value.as_bool().ok_or_else(|| D::Error::custom("preference values must be booleans"))?;
            match key.as_str() {
                "showInDiscover" => preferences.show_in_discover = Some(value),
                "showComments" => preferences.show_comments = Some(value),
                "showMentions" => preferences.show_mentions = Some(value),
                "showRecommends" => preferences.show_recommends = Some(value),
                _ => return Err(D::Error::custom(format!("unknown post preference {key}"))),
            }
        }
        Ok(preferences)
    }
}

impl Preferences {
    /// Native defaults are true. Explicit true and omitted are equal public
    /// meaning, but their source/wire spelling stays intact.
    pub fn meaning(&self) -> Value {
        json!({"showInDiscover":self.show_in_discover.unwrap_or(true),
            "showComments":self.show_comments.unwrap_or(true),
            "showMentions":self.show_mentions.unwrap_or(true),
            "showRecommends":self.show_recommends.unwrap_or(true)})
    }

    /// Native records persist only Discover's false opt-out.
    pub fn record(&self) -> Value {
        let mut value = serde_json::to_value(self).expect("preferences JSON");
        if self.show_in_discover != Some(false) {
            value.as_object_mut().expect("mapping").remove("showInDiscover");
        }
        value
    }
}
