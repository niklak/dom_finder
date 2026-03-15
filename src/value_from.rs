use super::errors::ValueConversionError;
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
            type Error = ValueConversionError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                if let Value::$variant(v) = value {
                    Ok(v.clone())
                } else {
                    Err(ValueConversionError {
                        expected: stringify!($variant),
                    })
                }
            }
        }
        impl TryFrom<&Value> for $inner_type {
            type Error = ValueConversionError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                if let Value::$variant(v) = value {
                    Ok(v.clone())
                } else {
                    Err(ValueConversionError {
                        expected: stringify!($variant),
                    })
                }
            }
        }
    };
}
macro_rules! impl_try_from_to_vec {
    ($variant:ident, $rust_type:ty) => {
        impl TryFrom<Value> for Vec<$rust_type> {
            type Error = ValueConversionError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::Array(val) => val
                        .iter()
                        .map(|v| {
                            <$rust_type>::try_from(v).map_err(|_| ValueConversionError {
                                expected: concat!("Array<", stringify!($variant), ">"),
                            })
                        })
                        .collect(),
                    _ => Err(ValueConversionError {
                        expected: concat!("Array<", stringify!($variant), ">"),
                    }),
                }
            }
        }

        impl TryFrom<&Value> for Vec<$rust_type> {
            type Error = ValueConversionError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                match value {
                    Value::Array(val) => val
                        .iter()
                        .map(|v| {
                            <$rust_type>::try_from(v).map_err(|_| ValueConversionError {
                                expected: concat!("Array<", stringify!($variant), ">"),
                            })
                        })
                        .collect(),
                    _ => Err(ValueConversionError {
                        expected: concat!("Array<", stringify!($variant), ">"),
                    }),
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

impl_try_from_to_vec!(Int, i64);
impl_try_from_to_vec!(Float, f64);
impl_try_from_to_vec!(Bool, bool);
impl_try_from_to_vec!(String, String);

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = ValueConversionError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::String(val) = value {
            Ok(val.as_str())
        } else {
            Err(ValueConversionError { expected: "String" })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_from_value_to_vec_i64() {
        let source = vec![1, 2, 3];
        let val = Value::from_iter(source.clone());
        let v: Vec<i64> = val.try_into().unwrap();
        assert_eq!(source, v);
    }

    #[test]
    fn try_from_value_to_vec_f64() {
        let source = vec![1.0, 2.0, 3.0];
        let val = Value::from_iter(source.clone());
        let v: Vec<f64> = val.try_into().unwrap();
        assert_eq!(source, v);
    }

    #[test]
    fn try_from_value_to_vec_string() {
        let source = vec!["1", "2", "3"];
        let val = Value::from_iter(source.clone());

        let v: Vec<String> = val.try_into().unwrap();
        assert_eq!(source, v);
    }

    #[test]
    fn try_from_value_to_vec_bool() {
        let source = vec![true, false, false];
        let val = Value::from_iter(source.clone());

        let v: Vec<bool> = val.try_into().unwrap();
        assert_eq!(source, v);
    }
    #[test]
    fn try_from_value_to_vec_wrong_type() {
        let source = vec![1.0, 2.0, 3.0];
        let val = Value::from_iter(source.clone());
        let val_ref = &val;
        let res: Result<Vec<i64>, _> = val_ref.try_into();
        let err = res.unwrap_err();
        assert_eq!(err.expected, "Array<Int>");

        let res: Result<i64, _> = val_ref.try_into();
        let err = res.unwrap_err();
        assert_eq!(err.expected, "Int");
    }

    #[test]
    fn try_from_value_wrong_type() {
        let source = 1.0;
        let val = Value::from(source);
        let res: Result<Vec<i64>, _> = val.try_into();
        let err = res.unwrap_err();
        assert_eq!(err.expected, "Array<Int>");
    }
}
