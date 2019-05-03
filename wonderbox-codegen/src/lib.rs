extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn resolve_dependencies(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr);
    println!("item: \"{:#?}\"", item);
    let some_impl: TokenStream = quote!(
        impl BarImpl {
            fn generated_fn(&self) {

            }
        }
    )
    .into();
    item.extend(some_impl);
    item
}
