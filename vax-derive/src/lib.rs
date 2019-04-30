extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn resolvable(attr: TokenStream, item: TokenStream) -> TokenStream {
    quote!().into()
}

#[proc_macro_attribute]
pub fn implementation(attr: TokenStream, item: TokenStream) -> TokenStream {
    quote!().into()
}
