use std::convert::From;
use std::iter::FromIterator;

use hashbrown::HashMap;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use std::hash::BuildHasherDefault;

///Value is a enum that can be used to store any type of data
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Object(InnerMap),
    Null,
}

pub type InnerMap = HashMap<String, Value, BuildHasherDefault<FxHasher>>;

impl From<i64> for Value {
    fn from(item: i64) -> Self {
        Self::Int(item)
    }
}

impl From<f64> for Value {
    fn from(item: f64) -> Self {
        Self::Float(item)
    }
}

impl From<bool> for Value {
    fn from(item: bool) -> Self {
        Self::Bool(item)
    }
}

impl From<InnerMap> for Value {
    fn from(item: InnerMap) -> Self {
        Self::Object(item)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(item: &'a str) -> Self {
        Self::String(item.to_string())
    }
}

impl From<String> for Value {
    fn from(item: String) -> Self {
        Self::String(item)
    }
}

impl FromIterator<Value> for Value {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Self::Array(iter.into_iter().collect())
    }
}

impl FromIterator<(String, Value)> for Value {
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Self::Object(iter.into_iter().collect())
    }
}

impl FromIterator<i64> for Value {
    fn from_iter<I: IntoIterator<Item = i64>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Value::from).collect())
    }
}

impl FromIterator<f64> for Value {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Value::from).collect())
    }
}

impl FromIterator<bool> for Value {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Value::from).collect())
    }
}

impl<'a> FromIterator<&'a str> for Value {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Value::from).collect())
    }
}

impl Value {
    ///Returns true if the value inner representation is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Null => true,
            Self::String(val) => val.is_empty(),
            Self::Array(val) => val.is_empty(),
            Self::Object(val) => val.is_empty(),
            Self::Int(val) => *val == 0,
            Self::Float(val) => *val == 0.0,
            _ => false,
        }
    }

    pub fn get(&self, path: &str) -> Option<&Value> {
        let paths = path.splitn(2, '.').collect::<Vec<&str>>();
        let key = paths[0];

        match self {
            Self::Object(val) => val
                .get(key)
                .and_then(|v| {
                    if paths.len() > 1 {
                        v.get(paths[1])
                    } else {
                        Some(v)
                    }
                })
                .or_else(|| if paths.len() > 1 { None } else { val.get(key) }),
            _ => None,
        }
    }
}
