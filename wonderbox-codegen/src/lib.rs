extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, AttributeArgs, FnArg, ImplItem, Item,
    ItemImpl, ReturnType,
};

#[proc_macro_attribute]
pub fn resolve_dependencies(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let attr = parse_macro_input!(attr as AttributeArgs);

    let item = parse_item_impl(item);

    validate_item_impl(&item);

    let self_ty = &item.self_ty;

    let constructors = parse_constructors(&item);

    if constructors.len() != 1 {
        panic!("Expected one constructor, found {}", constructors.len());
    }

    let constructor_args = constructors.first().unwrap();

    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    TokenStream::from(quote! {
        #item

        impl #impl_generics wonderbox::internal::AutoResolvable for #self_ty #type_generics #where_clause {
             fn resolve(container: &wonderbox::Container) -> Option<Self> {
                unimplemented!()
             }
        }
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

fn parse_constructors(item_impl: &ItemImpl) -> Vec<&FunctionArguments> {
    item_impl
        .items
        .iter()
        .filter_map(|impl_item| match impl_item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .map(|method| &method.sig.decl)
        .filter(|declaration| match &declaration.output {
            ReturnType::Default => false,
            ReturnType::Type(_, type_) => type_ == &item_impl.self_ty,
        })
        .map(|declaration| &declaration.inputs)
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
        .collect()
}

const ATTRIBUTE_NAME: &str = "#[resolve_dependencies]";