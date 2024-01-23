use super::value::Value;


impl From<Value> for Option<String> {
    fn from(value: Value) -> Self {
        match value {
            Value::String(val) => Some(val),
            _ => None,
        }
    }
}

impl <'a>From<&'a Value> for Option<&'a str> {
    fn from(value: &'a Value) -> Self {
        match value {
            Value::String(val) => Some(val),
            _ => None,
        }
    }
}

impl From<& Value> for Option<String> {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(val) => Some(val.clone()),
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

impl From<&Value> for Option<i64> {
    fn from(value: &Value) -> Self {
        match value {
            Value::Int(val) => Some(*val),
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

impl From<&Value> for Option<f64> {
    fn from(value: &Value) -> Self {
        match value {
            Value::Float(val) => Some(*val),
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

impl From<&Value> for Option<bool> {
    fn from(value: &Value) -> Self {
        match value {
            Value::Bool(val) => Some(*val),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<String>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<&Value> for Option<Vec<String>> {
    fn from(value: &Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<i64>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<&Value> for Option<Vec<i64>> {
    fn from(value: &Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<f64>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<&Value> for Option<Vec<f64>> {
    fn from(value: &Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<Value> for Option<Vec<bool>> {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

impl From<&Value> for Option<Vec<bool>> {
    fn from(value: &Value) -> Self {
        match value {
            Value::Array(val) => val.iter().map(|v| v.into()).collect(),
            _ => None,
        }
    }
}

// TODO: add From<Value> for Option<HashMap<String, String>> and so on.
