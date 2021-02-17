use super::{Owner, StdOption, Zc};

pub struct Option<O, T>(Zc<O, StdOption<T>>)
where
    O: Owner;

impl<O, T> From<Zc<O, StdOption<T>>> for Option<O, T>
where
    O: Owner,
{
    fn from(zc: Zc<O, StdOption<T>>) -> Self {
        Self(zc)
    }
}

impl<O, T> Option<O, T>
where
    O: Owner,
{
    /// Returns `true` if the option is [`Some<T>`].
    pub fn is_some(&self) -> bool {
        self.as_inner().value.is_some()
    }

    /// Returns `true` if the option is [`None`].
    pub fn is_none(&self) -> bool {
        self.as_inner().value.is_none()
    }

    pub fn unwrap(self) -> Zc<O, T> {
        unsafe { self.0.map_unchecked(StdOption::unwrap) }
    }

    pub fn as_inner(&self) -> &Zc<O, StdOption<T>> {
        &self.0
    }

    pub fn into_inner(self) -> Zc<O, StdOption<T>> {
        self.0
    }
}
