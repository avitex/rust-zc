extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Dependant)]
pub fn derive_dependant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = input.ident;
    let lifetime_count = input.generics.lifetimes().count();
    let expanded = if lifetime_count == 0 {
        quote! {
            unsafe impl<'a> zc::Dependant<'a> for #struct_ident {
                type Static = #struct_ident;

                unsafe fn transmute_into_static(self) -> Self::Static {
                    self
                }
            }
        }
    } else if lifetime_count == 1 {
        quote! {
            unsafe impl<'a> zc::Dependant<'a> for #struct_ident<'a> {
                type Static = #struct_ident<'static>;

                unsafe fn transmute_into_static(self) -> Self::Static {
                    core::mem::transmute(self)
                }
            }
        }
    } else {
        let message = format!(
            "{} lifetimes on `{}` when only a single is valid on a `zc::Dependant`",
            lifetime_count, struct_ident
        );
        quote_spanned! { input.generics.span() => compile_error!(#message); }
    };
    TokenStream::from(expanded)
}
