extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn resolve_dependencies(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr);
    println!("item: \"{:#?}\"", item);
    let some_impl: TokenStream = quote!(
        impl BarImpl {
            fn generated_fn(&self) {

            }
        }
    )
    .into();

    concat_token_streams(item, some_impl)
}

fn concat_token_streams(first_stream: TokenStream, second_stream: TokenStream) -> TokenStream {
    let mut stream = TokenStream::new();
    stream.extend(first_stream);
    stream.extend(second_stream);
    stream
}
