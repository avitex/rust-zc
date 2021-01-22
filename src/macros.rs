/// Convenience macro for constructing a [`Zc`] type via a [`Dependant`]'s
/// [`From`].
///
/// See [`Zc::new()`] for an example.
///
/// This macro creates an intermediate function to annotate the lifetime
/// required for the `Construct` trait as the compiler is not smart enough yet
/// to infer it for us. See issues [22340] and [70263].
///
/// [22340]: https://github.com/rust-lang/rust/issues/22340
/// [70263]: https://github.com/rust-lang/rust/issues/70263
#[macro_export]
macro_rules! from {
    ($owner:expr, $dependant:ident, $target:ty) => {{
        fn _new_fn(arg: &$target) -> $dependant<'_> {
            $dependant::from(arg)
        }
        zc::Zc::new($owner, _new_fn)
    }};
}

/// Convenience macro for constructing a [`Zc`] type via a [`Dependant`]'s
/// [`TryFrom`].
///
/// See [`Zc::try_new()`] for an example.
///
/// This macro creates an intermediate function to annotate the lifetime
/// required for the `TryConstruct` trait as the compiler is not smart enough
/// yet to infer it for us. See issues [22340] and [70263].
///
/// [22340]: https://github.com/rust-lang/rust/issues/22340
/// [70263]: https://github.com/rust-lang/rust/issues/70263
/// [`TryFrom`]: core::convert::TryFrom
#[macro_export]
macro_rules! try_from {
    ($owner:expr, $dependant:ident, $target:ty) => {{
        fn _new_fn(
            arg: &$target,
        ) -> Result<$dependant<'_>, <$dependant as core::convert::TryFrom<&$target>>::Error>
        {
            <$dependant as core::convert::TryFrom<&$target>>::try_from(arg)
        }
        zc::Zc::try_new($owner, _new_fn)
    }};
}

/// Convenience macro for mapping a [`Zc`] type.
///
/// See [`Zc::map()`] for an example.
///
/// This macro creates an intermediate function to annotate the lifetime
/// required for the `Map` trait as the compiler is not smart enough yet
/// to infer it for us. See issues [22340] and [70263].
///
/// [22340]: https://github.com/rust-lang/rust/issues/22340
/// [70263]: https://github.com/rust-lang/rust/issues/70263
#[macro_export]
macro_rules! map {
    ($zc:expr, $to:ident, $from:ident, |$arg:ident| $body:expr) => {{
        fn _map_fn<'a>($arg: $from<'a>) -> $to<'a> {
            $body
        }
        $zc.map(_map_fn)
    }};
    ($zc:expr, $from:ident, $to:ident, $fn:expr) => {{
        fn _map_fn(from: $from<'_>) -> $to<'_> {
            let inner = $fn;
            (inner)(from)
        }
        $zc.map(_map_fn)
    }};
}
