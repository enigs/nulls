mod conversions;
mod impls;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Null<T> {
    Undefined,
    Null,
    Value(T),
}

impl<T> Default for Null<T> {
    fn default() -> Self {
        Self::Undefined
    }
}

