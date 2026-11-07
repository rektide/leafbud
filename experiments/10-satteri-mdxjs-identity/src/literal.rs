//! Reused from 09's literal gate; independent copy avoids its old Serde/SWC pins.
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Number, Value};
use std::fmt;

struct Literal(Value);
impl<'de> Deserialize<'de> for Literal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = Literal;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON literal with unique object keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Literal, E> {
                Ok(Literal(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Literal, E> {
                Ok(Literal(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Literal, E> {
                Ok(Literal(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Literal, E> {
                Number::from_f64(v)
                    .map(|n| Literal(Value::Number(n)))
                    .ok_or_else(|| E::custom("non-finite number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Literal, E> {
                Ok(Literal(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Literal, E> {
                Ok(Literal(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Literal, A::Error> {
                let mut values = Vec::new();
                while let Some(Literal(v)) = seq.next_element()? {
                    values.push(v);
                }
                Ok(Literal(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Literal, A::Error> {
                let mut values = Map::new();
                while let Some((key, Literal(v))) = map.next_entry::<String, Literal>()? {
                    if values.insert(key.clone(), v).is_some() {
                        return Err(de::Error::custom(format!("duplicate object key: {key}")));
                    }
                }
                Ok(Literal(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(JsonVisitor)
    }
}

pub fn parse(value: &str) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(value);
    let Literal(value) = Literal::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}
