#![feature(custom_attribute)]

use core::any::TypeId;
use maplit::hashmap;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

fn main() {
    let container = Container::new();

    let _string: Rc<String> = container.resolve_shared();

    println!("Resolved a string!");

    let _foo: Rc<dyn Foo> = container.resolve_shared();

    println!("Resolved a dyn Foo!");
}

trait Resolvable<T>
where
    T: ?Sized,
{
    fn resolve_shared(&self) -> Rc<T>;
}

struct Container {
    shared_items: HashMap<TypeId, Box<dyn Any>>,
}

impl Container {
    fn new() -> Self {
        let mut shared_items: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
        shared_items.insert(TypeId::of::<String>(), Box::new(Rc::new(String::new())));
        shared_items.insert(
            TypeId::of::<Foo>(),
            Box::new(Rc::new(FooImpl::new()) as Rc<dyn Foo>),
        );
        Self { shared_items }
    }
}

impl Resolvable<String> for Container {
    fn resolve_shared(&self) -> Rc<String> {
        let type_id = TypeId::of::<String>();
        let resolvable_type = self
            .shared_items
            .get(&type_id)
            .expect("No registered implementations of type String found");
        resolvable_type
            .downcast_ref::<Rc<String>>()
            .unwrap()
            .clone()
    }
}

impl Resolvable<Foo> for Container {
    fn resolve_shared(&self) -> Rc<dyn Foo> {
        let type_id = TypeId::of::<Foo>();
        let resolvable_type = self
            .shared_items
            .get(&type_id)
            .expect("No registered implementations of type Foo found");
        resolvable_type
            .downcast_ref::<Rc<dyn Foo>>()
            .unwrap()
            .clone()
    }
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
