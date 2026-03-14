use std::convert::From;
use std::iter::FromIterator;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

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

pub type InnerMap = HashMap<String, Value>;

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
            _ => false,
        }
    }

    pub fn from_path(&self, path: &str) -> Option<Value> {
        let mut parts = path.splitn(2, '.');
        let head = parts.next()?;
        let tail = parts.next();

        match self {
            Self::Object(obj) => {
                let v = obj.get(head)?;
                match tail {
                    Some(rest) => v.from_path(rest),
                    None => Some(v.clone()),
                }
            }

            Self::Array(arr) => {
                if head == "#" {
                    match tail {
                        None => Some(Value::Int(arr.len() as i64)),
                        Some(rest) => {
                            let values = arr.iter().filter_map(|v| v.from_path(rest));
                            Some(Value::from_iter(values))
                        }
                    }
                } else {
                    let index = head.parse::<usize>().ok()?;
                    let v = arr.get(index)?;
                    match tail {
                        Some(rest) => v.from_path(rest),
                        None => Some(v.clone()),
                    }
                }
            }
            _ => None,
        }
    }
}
