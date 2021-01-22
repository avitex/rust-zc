use crate::{Dependant, DependantWithLifetime};

pub unsafe trait Construct<'o, Owned>: Sized
where
    Owned: ?Sized,
{
    type Dependant: Dependant + 'static;

    unsafe fn construct(self, owned: &'o Owned) -> Self::Dependant;
}

unsafe impl<'o, Owned, Dependant, F> Construct<'o, Owned> for F
where
    Owned: ?Sized + 'o,
    Dependant: DependantWithLifetime<'o>,
    F: FnOnce(&'o Owned) -> Dependant,
{
    type Dependant = Dependant::Static;

    unsafe fn construct(self, owned: &'o Owned) -> Self::Dependant {
        (self)(owned).erase_lifetime()
    }
}

pub unsafe trait TryConstruct<'o, Owned>: Sized
where
    Owned: ?Sized,
{
    type Error: 'static;
    type Dependant: Dependant + 'static;

    unsafe fn try_construct(self, owned: &'o Owned) -> Result<Self::Dependant, Self::Error>;
}

unsafe impl<'o, Owned, Dependant, Error, F> TryConstruct<'o, Owned> for F
where
    Error: 'static,
    Owned: ?Sized + 'o,
    Dependant: DependantWithLifetime<'o>,
    F: FnOnce(&'o Owned) -> Result<Dependant, Error> + 'static,
{
    type Error = Error;
    type Dependant = Dependant::Static;

    unsafe fn try_construct(self, owned: &'o Owned) -> Result<Self::Dependant, Self::Error> {
        (self)(owned).map(|d| d.erase_lifetime())
    }
}

pub unsafe trait Map<'t, From>: Sized
where
    From: Dependant + 't,
{
    type Into: Dependant + 't;

    unsafe fn map(self, from: From::Static) -> Self::Into;
}

unsafe impl<'t, M, From, Into> Map<'t, From> for M
where
    M: FnOnce(From) -> Into,
    From: DependantWithLifetime<'t>,
    Into: DependantWithLifetime<'t>,
{
    type Into = Into;

    unsafe fn map(self, from: From::Static) -> Self::Into {
        (self)(From::hydrate_lifetime(from))
    }
}
