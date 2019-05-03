extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, AttributeArgs, FnArg, ImplItem, Item,
    ItemImpl,
};

#[proc_macro_attribute]
pub fn resolve_dependencies(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let attr = parse_macro_input!(attr as AttributeArgs);

    let item = parse_item_impl(item);

    validate_item_impl(&item);

    let self_ty = &item.self_ty;

    let constructors = parse_constructors(&item);

    TokenStream::from(quote! {
        #item
    })
}

fn parse_item_impl(item: Item) -> ItemImpl {
    match item {
        Item::Impl(item_impl) => item_impl,
        _ => panic!("{} needs to be placed over an impl block", ATTRIBUTE_NAME),
    }
}

fn validate_item_impl(item_impl: &ItemImpl) {
    assert!(
        item_impl.trait_.is_none(),
        "{} must be placed over a direct impl, not a trait impl",
        ATTRIBUTE_NAME
    )
}

type FunctionArguments = Punctuated<FnArg, Comma>;

fn parse_constructors(item_impl: &ItemImpl) -> impl Iterator<Item = &FunctionArguments> {
    item_impl
        .items
        .iter()
        .filter_map(|impl_item| match impl_item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .map(|method| &method.sig.decl.inputs)
        .filter(|inputs| {
            let first_input = inputs.first();
            match first_input {
                Some(first_arg) => match first_arg.value() {
                    FnArg::SelfRef(_) | FnArg::SelfValue(_) => false,
                    _ => true,
                },
                None => false,
            }
        })
}

const ATTRIBUTE_NAME: &str = "#[resolve_dependencies]";
