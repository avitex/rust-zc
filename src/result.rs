use core::convert::identity;
use core::fmt::Debug;

use super::{Owner, StdOption, StdResult, Zc};

pub struct Result<O, Ok, Err>(Zc<O, StdResult<Ok, Err>>)
where
    O: Owner;

impl<O, Ok, Err> From<Zc<O, StdResult<Ok, Err>>> for Result<O, Ok, Err>
where
    O: Owner,
{
    fn from(zc: Zc<O, StdResult<Ok, Err>>) -> Self {
        Self(zc)
    }
}

impl<O, Ok, Err> Result<O, Ok, Err>
where
    O: Owner,
{
    /// Returns `true` if the result is [`Ok`].
    pub fn is_ok(&self) -> bool {
        self.as_inner().value.is_ok()
    }

    /// Returns `true` if the result is [`Err`].
    pub fn is_err(&self) -> bool {
        self.as_inner().value.is_err()
    }

    /// Converts from `Zc<O, Result<Ok, Err>>` to [`StdOption<Zc<O, Ok>>`].
    ///
    /// Converts `self` into an [`StdOption<Zc<O, Ok>>`], consuming `self`,
    /// and discarding the error, if any.
    pub fn ok(self) -> StdOption<Zc<O, Ok>> {
        unsafe { self.into_inner().try_map_unchecked(identity).ok() }
    }

    /// Converts from `Zc<O, Result<Ok, Err>>` to [`StdOption<Zc<O, Err>>`].
    ///
    /// Converts `self` into an [`StdOption<Zc<O, Err>>`], consuming `self`,
    /// and discarding the success value, if any.
    pub fn err(self) -> StdOption<Zc<O, Err>> {
        unsafe {
            self.into_inner()
                .try_map_unchecked(|value| match value {
                    Ok(_) => Err(()),
                    Err(err) => Ok(err),
                })
                .ok()
        }
    }

    pub fn unwrap(self) -> Zc<O, Ok>
    where
        Err: Debug,
    {
        unsafe { self.0.map_unchecked(StdResult::unwrap) }
    }

    ///
    pub fn unwrap_map_err<F, E>(self, f: F) -> StdResult<Zc<O, Ok>, E>
    where
        Err: Debug,
        F: FnOnce(&dyn Debug) -> E,
    {
        unsafe {
            self.into_inner().try_map_unchecked(|result| match result {
                Ok(ok) => Ok(ok),
                Err(err) => Err(f(&err)),
            })
        }
    }

    pub fn unwrap_err(self) -> Zc<O, Err>
    where
        Ok: Debug,
    {
        unsafe { self.into_inner().map_unchecked(StdResult::unwrap_err) }
    }

    pub fn as_inner(&self) -> &Zc<O, StdResult<Ok, Err>> {
        &self.0
    }

    pub fn into_inner(self) -> Zc<O, StdResult<Ok, Err>> {
        self.0
    }
}
