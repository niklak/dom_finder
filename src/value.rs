use std::convert::From;
use std::iter::FromIterator;

use hashbrown::HashMap;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use std::hash::BuildHasherDefault;

///Value is a enum that can be used to store any basic type of data
#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl From<Value> for Option<String> {
    fn from(value: Value) -> Self {
        match value {
            Value::String(val) => Some(val),
            _ => None,
        }
    }
}

impl From<Value> for Option<i64> {
    fn from(value: Value) -> Self {
        match value {
            Value::Int(val) => Some(val),
            _ => None,
        }
    }
}

impl From<Value> for Option<f64> {
    fn from(value: Value) -> Self {
        match value {
            Value::Float(val) => Some(val),
            _ => None,
        }
    }
}

impl From<Value> for Option<bool> {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(val) => Some(val),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<String>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.into_iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<i64>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.into_iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<f64>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.into_iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<bool>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.into_iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

// TODO: add From<Value> for Option<HashMap<String, String>> and so on.

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

    pub fn from_path(&self, path: &str) -> Option<Value> {
        let paths = path.splitn(2, '.').collect::<Vec<&str>>();

        match self {
            Self::Object(obj) => {
                let key = paths[0];
                obj.get(key).and_then(|v| {
                    if paths.len() > 1 {
                        v.from_path(paths[1])
                    } else {
                        Some(v.clone())
                    }
                })
            }
            Self::Array(val) => {
                if paths.len() == 1 && paths[0] == "#" {
                    return Some(Value::Int(val.len() as i64));
                } else if paths[0] == "#" {
                    let values = val.iter().filter_map(|v| v.from_path(paths[1]));
                    return Some(Self::from_iter(values));
                }

                let index = paths[0].parse::<usize>().ok()?;

                val.get(index).and_then(|v| {
                    if paths.len() > 1 {
                        v.from_path(paths[1])
                    } else {
                        Some(v.clone())
                    }
                })
            }
            _ => None,
        }
    }
}
