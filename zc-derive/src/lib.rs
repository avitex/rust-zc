extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Field, GenericParam, Ident, Lifetime,
    LifetimeDef,
};

#[proc_macro_derive(Guarded, attributes(zc))]
pub fn derive_guarded(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let derive_opts = match parse_derive_attrs(&input, false) {
        Ok(opts) => opts,
        Err(err) => return TokenStream::from(err),
    };
    let guarded_check = impl_guarded_check(&input, &derive_opts, false);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn _zc_guarded_check() {
                #guarded_check
            }
        }
        unsafe impl #impl_generics zc::Guarded for #name #ty_generics #where_clause {}
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(Dependant, attributes(zc))]
pub fn derive_dependant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime_count = input.generics.lifetimes().count();
    let derive_opts = match parse_derive_attrs(&input, true) {
        Ok(opts) => opts,
        Err(err) => return TokenStream::from(err),
    };
    let mut static_generics = input.generics.clone();
    let mut dependant_generics = input.generics.clone();
    let guarded_check = impl_guarded_check(&input, &derive_opts, !derive_opts.guarded_impl);
    let static_lifetime = Lifetime::new("'static", Span::call_site());
    let dependant_lifetime = if lifetime_count == 0 {
        let dependant_lifetime = Lifetime::new("'a", Span::call_site());
        dependant_generics.params.insert(
            0,
            GenericParam::Lifetime(LifetimeDef::new(dependant_lifetime.clone())),
        );
        dependant_lifetime
    } else if lifetime_count == 1 {
        let first_lifetime_mut = static_generics.lifetimes_mut().next().unwrap();
        let dependant_lifetime = first_lifetime_mut.lifetime.clone();
        first_lifetime_mut.lifetime = static_lifetime;
        dependant_lifetime
    } else {
        let message = format!(
            "{} lifetimes on `{}` when only a single is valid on a `zc::Dependant`",
            lifetime_count, name
        );
        let error = quote_spanned! { input.generics.span() => compile_error!(#message); };
        return TokenStream::from(error);
    };

    let impl_dependant_generics = dependant_generics.split_for_impl().0;
    let ty_generic_static = static_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut dependant_impl = quote! {
        unsafe impl #impl_dependant_generics zc::Dependant for #name #ty_generics #where_clause {
            type Static = #name #ty_generic_static;

            unsafe fn erase_lifetime(self) -> Self::Static {
                #guarded_check
                core::mem::transmute(self)
            }
        }

        unsafe impl #impl_dependant_generics zc::DependantWithLifetime<#dependant_lifetime> for #name #ty_generics #where_clause {}
    };
    if derive_opts.guarded_impl {
        dependant_impl.extend(quote! {
            unsafe impl #impl_generics zc::Guarded for #name #ty_generics #where_clause {}
        });
    }
    TokenStream::from(dependant_impl)
}

fn impl_guarded_check(input: &DeriveInput, opts: &DeriveOpts, skip: bool) -> TokenStream2 {
    if skip {
        return TokenStream2::new();
    }
    match &input.data {
        Data::Struct(v) => field_checks(opts, v.fields.iter()),
        Data::Enum(v) => field_checks(opts, v.variants.iter().flat_map(|v| v.fields.iter())),
        Data::Union(_) => {
            quote_spanned! { input.span() => compile_error!("deriving `zc::Guarded` is not supported for unions"); }
        }
    }
}

fn field_checks<'f>(opts: &DeriveOpts, fields: impl Iterator<Item = &'f Field>) -> TokenStream2 {
    let mut checks = TokenStream2::new();
    checks.extend(quote! {
        pub fn copy_check<T: Copy>() {};
        pub fn guarded_check<T: zc::Guarded>() {};
    });
    for field in fields {
        let field_ty = &field.ty;
        let field_opts = match parse_field_attrs(opts, field) {
            Ok(opts) => opts,
            Err(err) => return err,
        };
        checks.extend(match field_opts.guard {
            GuardType::Copy => quote! {
                copy_check::<#field_ty>();
            },
            GuardType::Default => quote! {
                guarded_check::<#field_ty>();
            },
        });
    }
    checks
}

#[derive(Copy, Clone)]
enum GuardType {
    Copy,
    Default,
}

///////////////////////////////////////////////////////////////////////////////
// DeriveOpts

struct DeriveOpts {
    guard: GuardType,
    guarded_impl: bool,
}

fn parse_derive_attrs(
    input: &DeriveInput,
    for_dependant: bool,
) -> Result<DeriveOpts, TokenStream2> {
    let zc_attr_ident = Ident::new("zc", Span::call_site());
    let zc_attrs = input
        .attrs
        .iter()
        .filter(|attr| attr.path.get_ident() == Some(&zc_attr_ident));

    let mut attrs = DeriveOpts {
        guard: GuardType::Default,
        guarded_impl: true,
    };

    for attr in zc_attrs {
        let attr_value = attr.tokens.to_string();

        if attr_value == "(unguarded)" {
            if !for_dependant {
                return Err(quote_spanned! {
                    attr.span() => compile_error!(
                        "attempting to disable `zc::Guarded` implementation while deriving `zc::Guarded`"
                    );
                });
            }
            attrs.guarded_impl = false;
        } else {
            attrs.guard = parse_guard_type(&attr, attr_value.as_str())?;
        }
    }

    Ok(attrs)
}

///////////////////////////////////////////////////////////////////////////////
// FieldOpts

struct FieldOpts {
    guard: GuardType,
}

fn parse_field_attrs(opts: &DeriveOpts, input: &Field) -> Result<FieldOpts, TokenStream2> {
    let zc_attr_ident = Ident::new("zc", Span::call_site());
    let zc_attrs = input
        .attrs
        .iter()
        .filter(|attr| attr.path.get_ident() == Some(&zc_attr_ident));

    let mut attrs = FieldOpts { guard: opts.guard };

    for attr in zc_attrs {
        attrs.guard = parse_guard_type(&attr, attr.tokens.to_string().as_str())?;
    }

    Ok(attrs)
}

fn parse_guard_type(attr: &Attribute, attr_value: &str) -> Result<GuardType, TokenStream2> {
    match attr_value {
        r#"(guard = "Copy")"# => Ok(GuardType::Copy),
        r#"(guard = "Default")"# => Ok(GuardType::Default),
        _ => Err(quote_spanned! { attr.span() => compile_error!("Unknown `zc` options"); }),
    }
}
