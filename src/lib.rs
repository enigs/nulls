use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sqlx::{Decode, Encode, Error, Postgres, Type, ValueRef};
use sqlx::encode::IsNull;
use sqlx::postgres::{PgTypeInfo, PgValueRef};
use sqlx::types::Json;
use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Null<T> {
    Undefined,
    Null,
    Value(T),
}

impl<T> Type<Postgres> for Null<T>
    where T: Type<Postgres>,
{
    fn type_info() -> PgTypeInfo {
        T::type_info()
    }
}

impl<'q, T> Encode<'q, Postgres> for Null<T>
    where T: Encode<'q, Postgres> + Type<Postgres>,
{
    fn encode_by_ref(&self, buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>) ->  Result<IsNull, Box<(dyn serde::ser::StdError + Send + Sync + 'static)>> {
        match self {
            Null::Value(ref value) => value.encode_by_ref(buf),
            Null::Undefined | Null::Null => Ok(IsNull::Yes),
        }
    }
}

impl<'r, T> Decode<'r, Postgres> for Null<T>
    where T: Decode<'r, Postgres> + Type<Postgres>,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        if value.is_null() {
            Ok(Null::Null)
        } else {
            T::decode(value).map(Null::Value)
        }
    }
}

impl<T: Display> Display for Null<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_str = match self {
            Null::Undefined => "Undefined".to_string(),
            Null::Null => "Null".to_string(),
            Null::Value(value) => format!("Value({})", value)
        };

        write!(f, "{}", variant_str)
    }
}

pub fn new<T>(value: T) -> Null<T> {
    Null::Value(value)
}

pub fn undefined<T>() -> Null<T> {
    Null::Undefined
}

pub fn null<T>() -> Null<T> {
    Null::Null
}

impl<T> Default for Null<T> {
    fn default() -> Self {
        Self::Undefined
    }
}

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

impl<T> Null<T> {
    pub const fn is_undefined(&self) -> bool {
        matches!(self, Null::Undefined)
    }

    pub const fn is_null(&self) -> bool {
        matches!(self, Null::Null)
    }

    pub const fn is_value(&self) -> bool {
        matches!(self, Null::Value(_))
    }

    pub const fn value(&self) -> Option<&T> {
        match self {
            Null::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn take(self) -> Option<T> {
        match self {
            Null::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn contains_value<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            Null::Value(y) => x == y,
            _ => false,
        }
    }

    pub fn contains<U>(&self, x: &Option<U>) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            Null::Value(y) => matches!(x, Some(v) if v == y),
            Null::Null => x.is_none(),
            Null::Undefined => false,
        }
    }

    pub fn map<U, F: FnOnce(Option<T>) -> Option<U>>(self, f: F) -> Null<U> {
        match self {
            Null::Value(v) => match f(Some(v)) {
                Some(v) => Null::Value(v),
                None => Null::Null,
            },
            Null::Null => match f(None) {
                Some(v) => Null::Value(v),
                None => Null::Null,
            },
            Null::Undefined => Null::Undefined,
        }
    }

    pub fn map_value<U, F: FnOnce(T) -> U>(self, f: F) -> Null<U> {
        match self {
            Null::Value(v) => Null::Value(f(v)),
            Null::Null => Null::Null,
            Null::Undefined => Null::Undefined,
        }
    }

    pub fn update_to(self, value: &mut Option<T>) {
        match self {
            Null::Value(new) => *value = Some(new),
            Null::Null => *value = None,
            Null::Undefined => {}
        };
    }
}