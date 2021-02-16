use core::convert::identity;
use core::fmt::Debug;
use core::result::Result as StdResult;

use super::{Owner, Zc};

pub type Result<O, Ok, Err> = Zc<O, StdResult<Ok, Err>>;

impl<O, Ok, Err> Result<O, Ok, Err>
where
    O: Owner,
{
    pub fn ok(self) -> Option<Zc<O, Ok>> {
        unsafe { self.try_map_unchecked(identity).ok() }
    }

    pub fn err(self) -> Option<Zc<O, Err>> {
        unsafe {
            self.try_map_unchecked(|value| match value {
                Ok(_) => Err(()),
                Err(err) => Ok(err),
            })
            .ok()
        }
    }

    pub fn is_ok(&self) -> bool {
        self.value.is_ok()
    }

    pub fn is_err(&self) -> bool {
        self.value.is_err()
    }

    pub fn unwrap(self) -> Zc<O, Ok>
    where
        Err: Debug,
    {
        unsafe { self.map_unchecked(|result| result.unwrap()) }
    }
}
