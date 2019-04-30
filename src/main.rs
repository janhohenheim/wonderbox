#![feature(custom_attribute)]

use core::any::TypeId;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

fn main() {
    let mut container = Container::new();

    container.register(Rc::new(String::new()));
    println!("Registered a string!");

    let _string: Rc<String> = container.resolve_shared::<String>();
    println!("Resolved a string!");

    container.register(Rc::new(FooImpl::new()) as Rc<dyn Foo>);
    println!("Registered a Foo!");

    let _foo: Rc<dyn Foo> = container.resolve_shared::<dyn Foo>();
    println!("Resolved a dyn Foo!");
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

    fn register<T>(&mut self, implementation: T) -> &mut Self
    where
        T: 'static,
    {
        self.shared_items
            .insert(TypeId::of::<T>(), Box::new(implementation));
        self
    }

    fn resolve_shared<T>(&self) -> Rc<T>
    where
        T: 'static + ?Sized,
    {
        let type_id = TypeId::of::<Rc<T>>();
        let resolvable_type = self
            .shared_items
            .get(&type_id)
            .expect("No registered implementations of type T found");
        resolvable_type.downcast_ref::<Rc<T>>().unwrap().clone()
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
