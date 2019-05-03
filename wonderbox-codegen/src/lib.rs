extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Item};

#[proc_macro_attribute]
pub fn resolve_dependencies(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let attr = parse_macro_input!(attr as AttributeArgs);

    //let item_impl = parse_item_impl(item);

    let some_impl = quote!(
        impl BarImpl {
            fn generated_fn(&self) {

            }
        }
    );

    TokenStream::from(quote! {
        #item

        #some_impl
    })
}

fn parse_item_impl(item: Item) -> syn::ItemImpl {
    match item {
        syn::Item::Impl(item_impl) => item_impl,
        _ => panic!("{} needs to be placed over an impl block", ATTRIBUTE_NAME),
    }
}

const ATTRIBUTE_NAME: &str = "#[resolve_dependencies]";
