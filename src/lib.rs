//! # `auto_ref`
//!
//! Attributes for replace reference parameter `&T` to `impl AsRef<T>` or `impl Borrow<T>`
//!
//! ## Usage
//!
//! ```rs
//! use auto_ref::{auto_ref, auto_borrow};
//!
//! #[auto_ref]
//! fn example1(s: &str, t: &mut str) {
//!     println!("{}, {}", s, t);
//! }
//!
//! #[auto_borrow]
//! fn example2(s: &str, t: &mut str) {
//!     println!("{}, {}", s, t);
//! }!
//!
//! #[auto_ref(s)]
//! #[auto_borrow(t)]
//! fn example3(s: &str, t: &str) {
//!     println!("{}, {}", s, t);
//! }
//! ```
//!
//! The above code is convert to
//!
//! ```rs
//! fn example1(s: impl ::core::convert::AsRef<str>, t: impl ::core::convert::AsMut<str>) {
//!     let s: &str = s.as_ref();
//!     let t: &mut str = t.as_mut();
//!     println!("{}, {}", s, t);
//! }
//!
//! fn example2(s: impl ::core::borrow::Borrow<str>, t: impl ::core::borrow::BorrowMut<str>) {
//!     let s: &str = s.as_borrow();
//!     let t: &mut str = t.as_borrow_mut();
//!     println!("{}, {}", s, t);
//! }
//!
//! fn example3(s: impl ::core::convert::AsRef<str>, t: impl ::core::borrow::Borrow<str>) {
//!     let t: &str = t.as_borrow();
//!     let s: &str = s.as_ref();
//!     println!("{}, {}", s, t);
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
