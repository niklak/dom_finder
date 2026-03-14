use super::value::{InnerMap, Value};

macro_rules! impl_value_extractors {
    ($variant:ident, $inner_type:ty, $as_method:ident, $to_method:ident) => {
        impl Value {
            #[inline]
            pub fn $as_method(&self) -> Option<&$inner_type> {
                if let Self::$variant(v) = self {
                    Some(v)
                } else {
                    None
                }
            }

            #[inline]
            pub fn $to_method(&self) -> Option<$inner_type> {
                if let Self::$variant(v) = self {
                    Some(v.clone())
                } else {
                    None
                }
            }
        }
        impl_try_from!($variant, $inner_type);
    };
}

macro_rules! impl_try_from {
    ($variant:ident, $inner_type:ty) => {
        impl TryFrom<Value> for $inner_type {
            type Error = &'static str;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                if let Value::$variant(v) = value {
                    Ok(v.clone())
                } else {
                    Err("Type mismatch")
                }
            }
        }
        impl TryFrom<&Value> for $inner_type {
            type Error = &'static str;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                if let Value::$variant(v) = value {
                    Ok(v.clone())
                } else {
                    Err("Type mismatch")
                }
            }
        }
    };
}
macro_rules! impl_try_from_to_vec {
    ($inner_type:ty) => {
        impl TryFrom<Value> for Vec<$inner_type> {
            type Error = &'static str;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::Array(val) => val
                        .iter()
                        .map(|v| <$inner_type>::try_from(v))
                        .collect(),
                    _ => Err("Type mismatch"),
                }
            }
        }
    };
}

impl_value_extractors!(Int, i64, as_i64, to_i64);
impl_value_extractors!(Float, f64, as_f64, to_f64);
impl_value_extractors!(Bool, bool, as_bool, to_bool);
impl_value_extractors!(String, String, as_string, to_string);
impl_value_extractors!(Array, Vec<Value>, as_array, to_array);
impl_value_extractors!(Object, InnerMap, as_object, to_object);

impl_try_from_to_vec!(i64);
impl_try_from_to_vec!(f64);
impl_try_from_to_vec!(bool);
impl_try_from_to_vec!(String);


// TODO: add From<Value> for Option<HashMap<String, String>> and so on.

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_from_value_to_vec_i64() {
        let source = vec![1, 2, 3];
        let val = Value::from_iter(source.clone().into_iter());
        let v: Vec<i64> = val.try_into().unwrap();
        assert_eq!(source, v);
    }
    
    #[test]
    fn try_from_value_to_vec_f64() {
        let source = vec![1.0, 2.0, 3.0];
        let val = Value::from_iter(source.clone().into_iter());
        let v: Vec<f64> = val.try_into().unwrap();
        assert_eq!(source, v);        
    }
    
    #[test]
    fn try_from_value_to_vec_string() {
        let source = vec!["1", "2", "3"];
        let val = Value::from_iter(source.clone().into_iter());

        let v: Vec<String> = val.try_into().unwrap();
        assert_eq!(source, v);
    }
    
    #[test]
    fn try_from_value_to_vec_bool() {
        let source = vec![true, false, false];
        let val = Value::from_iter(source.clone().into_iter());

        let v: Vec<bool> = val.try_into().unwrap();
        assert_eq!(source, v);
    }
    #[test]
    fn try_from_value_to_vec_wrong_type() {
        let source = vec![1.0, 2.0, 3.0];
        let val = Value::from_iter(source.clone().into_iter());
        let res: Result<Vec<i64>, _> = val.try_into();
        assert!(res.is_err())
        
    }
}
