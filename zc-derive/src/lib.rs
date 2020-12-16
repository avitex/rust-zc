extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, GenericParam, Ident, Lifetime, LifetimeDef};

#[proc_macro_derive(NoInteriorMut, attributes(zc))]
pub fn derive_no_interior_mut(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let derive_opts = match parse_derive_attrs(&input) {
        Ok(opts) => opts,
        Err(err) => return TokenStream::from(err),
    };
    let no_interior_mut_check = impl_no_interior_mut_check(&input, &derive_opts);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics for #name #ty_generics #where_clause {
            fn _zc_no_interior_mut_check() {
                #no_interior_mut_check
            }
        }
    };
    TokenStream::from(expanded)
}

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
    let no_interior_mut_check = impl_no_interior_mut_check(&input, &derive_opts);
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
        unsafe impl #impl_dependant_generics zc::Dependant<#dependant_lifetime> for #name #ty_generics #where_clause {
            type Static = #name #ty_generic_static;

            unsafe fn transmute_into_static(self) -> Self::Static {
                #no_interior_mut_check
                core::mem::transmute(self)
            }
        }
    };
    if derive_opts.no_interior_mut_impl {
        dependant_impl.extend(quote! {
            unsafe impl #impl_generics zc::NoInteriorMut for #name #ty_generics #where_clause {}
        });
    }
    TokenStream::from(dependant_impl)
}

fn impl_no_interior_mut_check(input: &DeriveInput, opts: &DeriveOpts) -> TokenStream2 {
    let mut checks = TokenStream2::new();
    if !opts.no_interior_mut_impl {
        return checks;
    }
    checks.extend(quote! {
        fn no_interior_mut_check<T: zc::NoInteriorMut>() {};
    });
    match &input.data {
        Data::Struct(v) => {
            for field in v.fields.iter() {
                let field_ty = &field.ty;
                checks.extend(quote! {
                    no_interior_mut_check::<#field_ty>();
                });
            }
            checks
        }
        Data::Enum(v) => {
            for field in v.variants.iter().flat_map(|v| v.fields.iter()) {
                let field_ty = &field.ty;
                checks.extend(quote! {
                    no_interior_mut_check::<#field_ty>();
                });
            }
            checks
        }
        Data::Union(_) => {
            quote_spanned! { input.span() => compile_error!("Deriving `zc::NoInteriorMut` is not supported for unions"); }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DeriveOpts

struct DeriveOpts {
    no_interior_mut_impl: bool,
}

fn parse_derive_attrs(input: &DeriveInput) -> Result<DeriveOpts, TokenStream2> {
    let zc_attr_ident = Ident::new("zc", Span::call_site());
    let zc_attrs = input
        .attrs
        .iter()
        .filter(|attr| attr.path.get_ident() == Some(&zc_attr_ident));

    let mut attrs = DeriveOpts {
        no_interior_mut_impl: true,
    };

    for attr in zc_attrs {
        let attr_value = attr.tokens.to_string();

        if attr_value == "(unguarded)" {
            attrs.no_interior_mut_impl = false;
        } else {
            return Err(
                quote_spanned! { attr.span() => compile_error!("Unknown derive options"); },
            );
        }
    }

    Ok(attrs)
}
