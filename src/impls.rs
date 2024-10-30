use crate::Null;

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