use crate::{EnumVariantData, RecError, RecObject, RecValue};
use serde::Serialize;

impl RecValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            RecValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            RecValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&RecObject> {
        match self {
            RecValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&RecValue> {
        match self {
            RecValue::Object(obj) => obj.fields.get(key),
            _ => None,
        }
    }
}

impl Serialize for RecValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RecValue::String(s) => serializer.serialize_str(s),
            RecValue::Int(i) => serializer.serialize_i64(*i),
            RecValue::Float(f) => serializer.serialize_f64(*f),
            RecValue::Bool(b) => serializer.serialize_bool(*b),
            RecValue::Null => serializer.serialize_none(),
            RecValue::Url(u) => serializer.serialize_str(u),
            RecValue::Socket(s) => serializer.serialize_str(s),
            RecValue::Pubkey(p) => serializer.serialize_str(p),
            RecValue::Array(arr) => arr.serialize(serializer),
            RecValue::Object(obj) => obj.fields.serialize(serializer),
            RecValue::EnumVariant {
                enum_name,
                variant,
                data,
            } => match data {
                EnumVariantData::Unit => {
                    serializer.serialize_str(&format!("{}.{}", enum_name, variant))
                }
                EnumVariantData::Tuple(values) => {
                    use serde::ser::SerializeMap;
                    let mut map = serializer.serialize_map(Some(2))?;
                    map.serialize_entry("variant", &format!("{}.{}", enum_name, variant))?;
                    map.serialize_entry("data", values)?;
                    map.end()
                }
                EnumVariantData::Struct(fields) => {
                    use serde::ser::SerializeMap;
                    let mut map = serializer.serialize_map(Some(2))?;
                    map.serialize_entry("variant", &format!("{}.{}", enum_name, variant))?;
                    map.serialize_entry("data", fields)?;
                    map.end()
                }
            },
        }
    }
}

pub trait RecDeserialize: Sized {
    fn from_rec(value: &RecValue) -> Result<Self, RecError>;
}

impl RecDeserialize for String {
    fn from_rec(value: &RecValue) -> Result<Self, RecError> {
        match value {
            RecValue::String(s) => Ok(s.clone()),
            _ => Err(RecError::TypeError {
                expected: "string".to_string(),
                actual: format!("{:?}", value),
            }),
        }
    }
}

impl RecDeserialize for i64 {
    fn from_rec(value: &RecValue) -> Result<Self, RecError> {
        match value {
            RecValue::Int(i) => Ok(*i),
            _ => Err(RecError::TypeError {
                expected: "int".to_string(),
                actual: format!("{:?}", value),
            }),
        }
    }
}

impl RecDeserialize for bool {
    fn from_rec(value: &RecValue) -> Result<Self, RecError> {
        match value {
            RecValue::Bool(b) => Ok(*b),
            _ => Err(RecError::TypeError {
                expected: "bool".to_string(),
                actual: format!("{:?}", value),
            }),
        }
    }
}

impl<T: RecDeserialize> RecDeserialize for Vec<T> {
    fn from_rec(value: &RecValue) -> Result<Self, RecError> {
        match value {
            RecValue::Array(arr) => arr
                .iter()
                .map(|v| T::from_rec(v))
                .collect::<Result<Vec<_>, _>>(),
            _ => Err(RecError::TypeError {
                expected: "array".to_string(),
                actual: format!("{:?}", value),
            }),
        }
    }
}

impl<T: RecDeserialize> RecDeserialize for Option<T> {
    fn from_rec(value: &RecValue) -> Result<Self, RecError> {
        match value {
            RecValue::Null => Ok(None),
            _ => T::from_rec(value).map(Some),
        }
    }
}
