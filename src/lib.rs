//! # `auto_ref`
//!
//! An attribute for replace reference parameter `&T` to `impl AsRef<T>`
//!
//! ## Usage
//!
//! ```rs
//! use auto_ref::auto_ref;
//!
//! #[auto_ref]
//! fn example(s: &str) {
//!     println!("{}", s);
//! }
//! ```
//!
//! The above code is convert to
//!
//! ```rs
//! fn example(s: impl ::code::convert::AsRef<str>) {
//!     let s = s.as_ref();
//!     println!("{}", s);
//! }
//! ```

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn auto_ref(attr: TokenStream, item: TokenStream) -> TokenStream {
    auto_ref_impl::auto_ref(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn auto_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    auto_ref_impl::auto_borrow(attr.into(), item.into()).into()
}
