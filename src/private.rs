use crate::Dependant;

pub unsafe trait Construct<'o, O: ?Sized>: Sized {
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
        (self)(owned).transmute_into_static()
    }
}

pub unsafe trait TryConstruct<'o, O: ?Sized>: Sized {
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
        (self)(owned).map(|d| d.transmute_into_static())
    }
}
