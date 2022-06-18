use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn auto_ref(attr: TokenStream, item: TokenStream) -> TokenStream {
    auto_ref_impl::auto_ref(attr.into(), item.into()).into()
}
