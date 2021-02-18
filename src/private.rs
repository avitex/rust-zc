use core::{mem, ptr};

use crate::Dependant;

unsafe fn erase_lifetime<'a, D: Dependant<'a>>(dependant: D) -> D::Static {
    let self_ptr: *const D = &dependant;
    let erased = ptr::read(self_ptr.cast::<D::Static>());
    mem::forget(dependant);
    erased
}

pub unsafe trait Construct<'o, O>: Sized
where
    O: ?Sized,
{
    type Dependant: Dependant<'static>;

    unsafe fn construct(self, owned: &'o O) -> Self::Dependant;
}

unsafe impl<'o, O, D, F> Construct<'o, O> for F
where
    O: ?Sized + 'o,
    D: Dependant<'o>,
    F: FnOnce(&'o O) -> D + 'static,
{
    type Dependant = D::Static;

    unsafe fn construct(self, owned: &'o O) -> Self::Dependant {
        erase_lifetime((self)(owned))
    }
}

pub unsafe trait TryConstruct<'o, O>: Sized
where
    O: ?Sized,
{
    type Error: 'static;
    type Dependant: Dependant<'static>;

    unsafe fn try_construct(self, owned: &'o O) -> Result<Self::Dependant, Self::Error>;
}

unsafe impl<'o, O, D, E, F> TryConstruct<'o, O> for F
where
    E: 'static,
    O: ?Sized + 'o,
    D: Dependant<'o>,
    F: FnOnce(&'o O) -> Result<D, E> + 'static,
{
    type Error = E;
    type Dependant = D::Static;

    unsafe fn try_construct(self, owned: &'o O) -> Result<Self::Dependant, Self::Error> {
        (self)(owned).map(|d| erase_lifetime(d))
    }
}
