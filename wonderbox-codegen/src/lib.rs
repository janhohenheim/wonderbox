#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

mod spanned;

use crate::spanned::SpannedUnstable;
use proc_macro::{Diagnostic, Level, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, AttributeArgs, FnArg,
    FnDecl, ImplItem, ImplItemMethod, Item, ItemImpl, ReturnType, Type,
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
        let error_message = format!("Expected one constructor, found {}", constructors.len());
        Diagnostic::spanned(item.span_unstable(), Level::Error, error_message).emit();
        return quote! {
            #item
        }
        .into();
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
        .filter_map(parse_method)
        .map(|method| &method.sig.decl)
        .filter(|declaration| returns_self(declaration, &item_impl.self_ty))
        .map(|declaration| &declaration.inputs)
        .filter(|inputs| has_no_self_parameter(inputs))
        .collect()
}

fn parse_method(impl_item: &ImplItem) -> Option<&ImplItemMethod> {
    match impl_item {
        ImplItem::Method(method) => Some(method),
        _ => None,
    }
}

fn returns_self(declaration: &FnDecl, explicit_self_type: &Type) -> bool {
    match &declaration.output {
        ReturnType::Default => false,
        ReturnType::Type(_, return_type) => {
            **return_type == generate_self_type() || **return_type == *explicit_self_type
        }
    }
}

fn has_no_self_parameter(inputs: &Punctuated<FnArg, Comma>) -> bool {
    let first_input = inputs.first();
    match first_input {
        Some(first_arg) => match first_arg.value() {
            FnArg::SelfRef(_) | FnArg::SelfValue(_) => false,
            _ => true,
        },
        None => true,
    }
}

fn generate_self_type() -> Type {
    parse_quote! {
        Self
    }
}

const ATTRIBUTE_NAME: &str = "#[resolve_dependencies]";
