#![feature(custom_attribute)]

use core::any::TypeId;
use maplit::hashmap;
use std::collections::HashMap;
use std::rc::Rc;

fn main() {
    let container = Container::new();

    let _string: Rc<String> = container.resolve_shared();

    let _foo: Rc<dyn Foo> = container.resolve_shared();
}

trait Resolvable<T>
where
    T: ?Sized,
{
    fn resolve_shared(&self) -> Rc<T>;
}

struct Container {
    shared_items: HashMap<TypeId, ResolvableType>,
}

impl Container {
    fn new() -> Self {
        Self {
            shared_items: hashmap! {
                TypeId::of::<String>() => ResolvableType::String(Rc::new(String::new())),
                TypeId::of::<Foo>() => ResolvableType::Foo(Rc::new(FooImpl::new()))
            },
        }
    }
}

impl Resolvable<String> for Container {
    fn resolve_shared(&self) -> Rc<String> {
        let type_id = TypeId::of::<String>();
        let resolvable_type = self
            .shared_items
            .get(&type_id)
            .expect("No registered implementations of type String found");
        match resolvable_type {
            ResolvableType::String(value) => value.clone(),
            _ => panic!(""),
        }
    }
}

impl Resolvable<Foo> for Container {
    fn resolve_shared(&self) -> Rc<dyn Foo> {
        let type_id = TypeId::of::<Foo>();
        let resolvable_type = self
            .shared_items
            .get(&type_id)
            .expect("No registered implementations of type Foo found");
        match resolvable_type {
            ResolvableType::Foo(value) => value.clone(),
            _ => panic!(""),
        }
    }
}

enum ResolvableType {
    String(Rc<String>),
    Foo(Rc<dyn Foo>),
}

#[resolvable]
trait Foo {}

#[implementationt(crate::Foo)]
struct FooImpl {}

impl FooImpl {
    fn new() -> Self {
        FooImpl {}
    }
}

impl Foo for FooImpl {}
