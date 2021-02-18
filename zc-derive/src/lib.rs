extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Field, GenericParam, Ident, Lifetime,
    LifetimeDef,
};

#[proc_macro_derive(Dependant, attributes(zc))]
pub fn derive_dependant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime_count = input.generics.lifetimes().count();
    let derive_opts = match parse_derive_attrs(&input) {
        Ok(opts) => opts,
        Err(err) => return TokenStream::from(err),
    };
    let mut static_generics = input.generics.clone();
    let mut dependant_generics = input.generics.clone();
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
    let field_checks = impl_field_checks(&input, &derive_opts, &dependant_lifetime);
    let impl_dependant_generics = dependant_generics.split_for_impl().0;
    let ty_generic_static = static_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let dependant_impl = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn _zc_field_checks() {
                #field_checks
            }
        }

        unsafe impl #impl_dependant_generics ::zc::Dependant<#dependant_lifetime> for #name #ty_generics #where_clause {
            type Static = #name #ty_generic_static;
        }
    };
    TokenStream::from(dependant_impl)
}

fn impl_field_checks(input: &DeriveInput, opts: &DeriveOpts, lifetime: &Lifetime) -> TokenStream2 {
    match &input.data {
        Data::Struct(v) => field_checks(opts, v.fields.iter(), lifetime),
        Data::Enum(v) => field_checks(
            opts,
            v.variants.iter().flat_map(|v| v.fields.iter()),
            lifetime,
        ),
        Data::Union(_) => {
            quote_spanned! { input.span() => compile_error!("deriving `zc::Dependant` is not supported for unions"); }
        }
    }
}

fn field_checks<'f>(
    opts: &DeriveOpts,
    fields: impl Iterator<Item = &'f Field>,
    lifetime: &Lifetime,
) -> TokenStream2 {
    let mut checks = TokenStream2::new();
    checks.extend(quote! {
        pub fn copy_check<'a, T: Copy + 'a>() {};
        pub fn dependant_check<'a, T: ::zc::Dependant<'a>>() {};
    });
    for field in fields {
        let field_ty = &field.ty;
        let field_opts = match parse_field_attrs(opts, field) {
            Ok(opts) => opts,
            Err(err) => return err,
        };
        checks.extend(match field_opts.guard {
            CheckType::Copy => quote! {
                copy_check::<#lifetime, #field_ty>();
            },
            CheckType::Default => quote! {
                dependant_check::<#lifetime, #field_ty>();
            },
        });
    }
    checks
}

#[derive(Copy, Clone)]
enum CheckType {
    Copy,
    Default,
}

///////////////////////////////////////////////////////////////////////////////
// DeriveOpts

struct DeriveOpts {
    check: CheckType,
}

fn parse_derive_attrs(input: &DeriveInput) -> Result<DeriveOpts, TokenStream2> {
    let zc_attr_ident = Ident::new("zc", Span::call_site());
    let zc_attrs = input
        .attrs
        .iter()
        .filter(|attr| attr.path.get_ident() == Some(&zc_attr_ident));

    let mut attrs = DeriveOpts {
        check: CheckType::Default,
    };

    for attr in zc_attrs {
        let attr_value = attr.tokens.to_string();

        attrs.check = parse_guard_type(&attr, attr_value.as_str())?;
    }

    Ok(attrs)
}

///////////////////////////////////////////////////////////////////////////////
// FieldOpts

struct FieldOpts {
    guard: CheckType,
}

fn parse_field_attrs(opts: &DeriveOpts, input: &Field) -> Result<FieldOpts, TokenStream2> {
    let zc_attr_ident = Ident::new("zc", Span::call_site());
    let zc_attrs = input
        .attrs
        .iter()
        .filter(|attr| attr.path.get_ident() == Some(&zc_attr_ident));

    let mut attrs = FieldOpts { guard: opts.check };

    for attr in zc_attrs {
        attrs.guard = parse_guard_type(&attr, attr.tokens.to_string().as_str())?;
    }

    Ok(attrs)
}

fn parse_guard_type(attr: &Attribute, attr_value: &str) -> Result<CheckType, TokenStream2> {
    match attr_value {
        r#"(check = "Copy")"# => Ok(CheckType::Copy),
        r#"(guard = "Default")"# => Ok(CheckType::Default),
        _ => Err(quote_spanned! { attr.span() => compile_error!("Unknown `zc` options"); }),
    }
}
