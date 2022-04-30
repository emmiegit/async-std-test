/*
 * lib.rs
 *
 * async-std-test - Alternate implementation of the async-std test macro
 * Copyright (c) 2022 Ammon Smith
 *
 * async-std-test is available free of charge under the terms of the MIT
 * License. You are free to redistribute and/or modify it under those
 * terms. It is distributed in the hopes that it will be useful, but
 * WITHOUT ANY WARRANTY. See the LICENSE file for more details.
 *
 */

#![forbid(unsafe_code, future_incompatible)]
#![deny(missing_debug_implementations, nonstandard_style)]

//! An alternate method of running `async fn` tests. Meant for use with [`async-std`].
//!
//! The only export in this crate is a procedural macro, [`async_test`].
//! It can be invoked as follows:
//!
//! ```ignore
//! #[async_test]
//! async fn my_test() -> std::io::Result<()> {
//!     assert_eq!(2 * 2, 4);
//!     Ok(())
//! }
//! ```
//!
//! [`async-std`]: https://docs.rs/async-std
//! [`async_test`]: ./attr.async_test.html

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Enables this test to be run in an `async fn`.
///
/// Requires that the test return [`Result`] with the error
/// type implementing [`Display`].
///
/// # Examples
///
/// ```ignore
/// #[async_test]
/// async fn my_test() -> std::io::Result<()> {
///     assert_eq!(2 * 2, 4);
///     Ok(())
/// }
/// ```
///
/// [`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
#[proc_macro_attribute]
pub fn async_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let vis = &input.vis;

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        #[::core::prelude::v1::test]
        #(#attrs)*
        #vis fn #name() #ret {
            async fn test_inner() #ret {
                #body
            }

            async_std::task::block_on(async {
                if let Err(error) = test_inner().await {
                    panic!("Error in test: {}", error);
                }
            })
        }
    };

    result.into()
}
