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

macro_rules! impl_from_type_to_value {
    ($inner_type:ty,$variant:ident) => {
        impl From<$inner_type> for Value {
            fn from(value: $inner_type) -> Self {
                Self::$variant(value)
            }
        }

        impl FromIterator<$inner_type> for Value {
            fn from_iter<I: IntoIterator<Item = $inner_type>>(iter: I) -> Self {
                Self::Array(iter.into_iter().map(Value::from).collect())
            }
        }
    };
}

impl_from_type_to_value!(i64, Int);
impl_from_type_to_value!(f64, Float);
impl_from_type_to_value!(bool, Bool);
impl_from_type_to_value!(InnerMap, Object);
impl_from_type_to_value!(String, String);

impl From<&str> for Value {
    fn from(item: &str) -> Self {
        Self::String(item.to_string())
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

impl<'a> FromIterator<&'a str> for Value {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        iter.into_iter().map(Value::from).collect()
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
