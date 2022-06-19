use std::{borrow::Cow, collections::HashSet};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::Parse, Block, FnArg, ItemFn, Pat, PatType, Signature, Token, Type};

struct Filters {
    idents: HashSet<Ident>,
}

impl Parse for Filters {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut idents = HashSet::new();
        while !input.is_empty() {
            idents.insert(input.parse()?);
            std::mem::drop(input.parse::<Token![,]>());
        }
        Ok(Self { idents })
    }
}

struct AutoRef {
    attr: TokenStream,
    item: TokenStream,
    ref_trait: TokenStream,
    mut_trait: TokenStream,
    ref_fn: TokenStream,
    mut_fn: TokenStream,
}

impl AutoRef {
    fn try_into_token_stream(self) -> syn::Result<TokenStream> {
        let Self {
            attr,
            item,
            ref_trait,
            mut_trait,
            ref_fn,
            mut_fn,
        } = self;
        let ItemFn {
            attrs,
            vis,
            sig,
            block,
        } = syn::parse2(item)?;
        let filters: Filters = syn::parse2(attr)?;
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
                    let PatType { attrs, pat, ty, .. } = &pat_type;
                    if let Pat::Ident(ident) = pat.as_ref() {
                        if !filters.idents.is_empty() && !filters.idents.contains(&ident.ident) {
                            return pat_type.to_token_stream();
                        }
                    }
                    if let Type::Reference(ty) = ty.as_ref() {
                        let lifetime = ty.lifetime.as_ref().map(|l| quote! { #l + });
                        let elem = &ty.elem;
                        let var = if let Pat::Ident(ident) = pat.as_ref() {
                            Cow::Borrowed(&ident.ident)
                        } else {
                            Cow::Owned(Ident::new(format!("__p{}", i).as_str(), Span::call_site()))
                        };
                        if ty.mutability.is_some() {
                            bindings.push(quote! {
                                let #pat: &mut #elem = #var.#mut_fn();
                            });
                            quote! {
                                #(#attrs)* mut #var: impl #lifetime #mut_trait<#elem>
                            }
                        } else {
                            bindings.push(quote! {
                                let #pat: &#elem = #var.#ref_fn();
                            });
                            quote! {
                                #(#attrs)* #var: impl #lifetime #ref_trait<#elem>
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
}

#[must_use]
pub fn auto_ref(attr: TokenStream, item: TokenStream) -> TokenStream {
    AutoRef {
        attr,
        item,
        ref_trait: quote!(::core::convert::AsRef),
        mut_trait: quote!(::core::convert::AsMut),
        ref_fn: quote!(as_ref),
        mut_fn: quote!(as_mut),
    }
    .try_into_token_stream()
    .unwrap_or_else(syn::Error::into_compile_error)
}

#[must_use]
pub fn auto_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    AutoRef {
        attr,
        item,
        ref_trait: quote!(::core::borrow::Borrow),
        mut_trait: quote!(::core::borrow::BorrowMut),
        ref_fn: quote!(borrow),
        mut_fn: quote!(borrow_mut),
    }
    .try_into_token_stream()
    .unwrap_or_else(syn::Error::into_compile_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_ref() {
        assert_eq!(
            format!(
                "{}",
                auto_ref(
                    quote! {},
                    quote! {
                        pub(in crate::a)
                        const async unsafe extern "C" fn test<'a, T: Sized>(s: &'a str, t: T, ...) -> ()
                        where T: 'a + Sized + Sized {
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
                    where T: 'a + Sized + Sized {
                        let s: &str = s.as_ref();
                        println!("{}", s);
                    }
                }
            )
        );
    }
}
