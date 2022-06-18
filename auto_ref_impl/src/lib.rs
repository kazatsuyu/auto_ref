use std::borrow::Cow;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Block, FnArg, ItemFn, Pat, PatType, Signature, Type};

#[allow(clippy::needless_pass_by_value)]
fn as_ref_impl(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = syn::parse2(item)?;
    let Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        ident,
        generics,
        inputs,
        variadic,
        output,
        ..
    } = sig;
    let where_clause = &generics.where_clause;
    let mut bindings = vec![];
    let inputs = inputs
        .into_iter()
        .enumerate()
        .map(|(i, input)| match input {
            FnArg::Typed(pat_type) => {
                if let Type::Reference(ty) = pat_type.ty.as_ref() {
                    let PatType { attrs, pat, .. } = &pat_type;
                    let lifetime = ty.lifetime.as_ref().map(|l| quote! { #l + });
                    let elem = &ty.elem;
                    let var = if let Pat::Ident(ident) = pat.as_ref() {
                        Cow::Borrowed(&ident.ident)
                    } else {
                        Cow::Owned(Ident::new(format!("__p{}", i).as_str(), Span::call_site()))
                    };
                    if ty.mutability.is_some() {
                        bindings.push(quote! {
                            let #pat = #var.as_mut();
                        });
                        quote! {
                            #(#attrs)* mut #var: impl #lifetime ::core::convert::AsMut<#elem>
                        }
                    } else {
                        bindings.push(quote! {
                            let #pat = #var.as_ref();
                        });
                        quote! {
                            #(#attrs)* #var: impl #lifetime ::core::convert::AsRef<#elem>
                        }
                    }
                } else {
                    pat_type.to_token_stream()
                }
            }
            input @ FnArg::Receiver(_) => input.to_token_stream(),
        });
    let Block { stmts, .. } = *block;
    Ok(quote! {
        #(#attrs)*
        #vis
        #constness
        #asyncness
        #unsafety
        #abi
        fn #ident #generics (#(#inputs,)* #variadic) #output
        #where_clause
        {
            #(#bindings)*
            #(#stmts)*
        }
    })
}

#[must_use]
pub fn auto_ref(attr: TokenStream, item: TokenStream) -> TokenStream {
    as_ref_impl(attr, item).unwrap_or_else(syn::Error::into_compile_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(
            format!(
                "{}",
                auto_ref(
                    quote! {},
                    quote! {
                        pub(in crate::a)
                        const async unsafe extern "C" fn test<'a, T: Sized>(s: &'a str, t: T, ...) -> ()
                        where T: Sized + Sized {
                            println!("{}", s);
                        }
                    }
                )
            ),
            format!(
                "{}",
                quote! {
                    pub(in crate::a)
                    const async unsafe extern "C" fn test<'a, T: Sized>(s: impl 'a + ::core::convert::AsRef<str>, t: T, ...) -> ()
                    where T: Sized + Sized {
                        let s = s.as_ref();
                        println!("{}", s);
                    }
                }
            )
        )
    }
}
