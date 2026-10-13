use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use std::fmt;

struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("unique-key structured data")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Unique, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| Unique(Value::Number(n)))
                    .ok_or_else(|| E::custom("invalid number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(Value::String(v.into())))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Unique, E> {
                Ok(Unique(Value::String(v)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_none<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Unique, A::Error> {
                let mut values = vec![];
                while let Some(Unique(v)) = seq.next_element()? {
                    if values.len() >= 100000 {
                        return Err(de::Error::custom("element limit"));
                    }
                    values.push(v);
                }
                Ok(Unique(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Unique, A::Error> {
                let mut values = Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("duplicate mapping key"));
                    }
                    let Unique(v) = map.next_value()?;
                    values.insert(key, v);
                }
                Ok(Unique(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}

pub fn parse(text: &str, json: bool) -> Result<Value, crate::model::AnalysisError> {
    let error = || {
        crate::model::AnalysisError::new(
            "UNSUPPORTED_CHANGE",
            "invalid, duplicate-key or unsupported structured data",
        )
    };
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for c in text.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quoted {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
            continue;
        }
        if !quoted && matches!(c, '[' | '{') {
            depth += 1;
            if depth > 64 {
                return Err(error());
            }
        }
        if !quoted && matches!(c, ']' | '}') {
            depth = depth.saturating_sub(1);
        }
    }
    if !json
        && text
            .lines()
            .any(|line| line.len() - line.trim_start().len() > 128)
    {
        return Err(error());
    }
    if json {
        let mut d = serde_json::Deserializer::from_str(text);
        let Unique(value) = Unique::deserialize(&mut d).map_err(|_| error())?;
        d.end().map_err(|_| error())?;
        Ok(value)
    } else {
        // YAML aliases/tags are outside this bounded MVP profile; never expand graphs.
        if text
            .lines()
            .any(|l| l.contains('&') || l.contains('*') || l.contains('!'))
        {
            return Err(error());
        }
        let Unique(value) = serde_yaml_ng::from_str::<Unique>(text).map_err(|_| error())?;
        Ok(value)
    }
}
