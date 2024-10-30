use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sqlx::{Error, types::Json};

use crate::Null;

impl<T: Serialize> Serialize for Null<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Null::Value(value) => value.serialize(serializer),
            _ => serializer.serialize_none(),
        }
    }
}

impl<'de, T> Deserialize<'de> for Null<T>
    where T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Null<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Value::deserialize(deserializer) {
            Ok(json) => match json {
                Value::Null => Ok(Null::Null),
                _ =>  {
                    if let Ok(value) = <T>::deserialize(json) {
                        return Ok(Null::Value(value));
                    }

                    Ok(Null::Undefined)
                }
            },
            Err(_) => Ok(Null::Undefined),
        }
    }
}

impl<T> From<Null<T>> for Option<Option<T>> {
    fn from(maybe_undefined: Null<T>) -> Self {
        match maybe_undefined {
            Null::Undefined => None,
            Null::Null => Some(None),
            Null::Value(value) => Some(Some(value)),
        }
    }
}

impl<T> From<Option<Option<T>>> for Null<T> {
    fn from(value: Option<Option<T>>) -> Self {
        match value {
            Some(Some(value)) => Self::Value(value),
            Some(None) => Self::Null,
            None => Self::Undefined,
        }
    }
}

impl<T> From<Option<T>> for Null<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::Undefined,
        }
    }
}


impl<T> From<Result<T, Error>> for Null<T> {
    fn from(value: Result<T, Error>) -> Self {
        match value {
            Ok(data) => Null::Value(data),
            _ => Null::Null
        }
    }
}

impl<T> From<Result<Json<T>, Error>> for Null<T> {
    fn from(value: Result<Json<T>, Error>) -> Self {
        match value {
            Ok(data) => Null::Value(data.0),
            _ => Null::Null
        }
    }
}


