#![feature(specialization)]

use core::any::TypeId;
use std::any::Any;
use std::collections::HashMap;

#[derive(Default)]
struct Container {
    registered_types: HashMap<TypeId, Box<dyn Any>>,
}

impl Container {
    fn new() -> Self {
        Self::default()
    }

    fn register<ImplementationType, Value>(
        &mut self,
        implementation: ImplementationType,
    ) -> &mut Self
    where
        ImplementationType: 'static + Implementation<Value>,
        Value: 'static,
    {
        self.registered_types
            .insert(TypeId::of::<Value>(), Box::new(Box::new(implementation)));
        self
    }

    fn resolve<T>(&self) -> Option<T>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let resolvable_type = self.registered_types.get(&type_id)?;
        let implementation = resolvable_type
            .downcast_ref::<Box<dyn Implementation<T>>>()
            .expect("Internal error: Couldn't downcast stored type to resolved type");
        let value: T = Implementation::get_value(implementation.as_ref(), &self);
        Some(value)
    }
}

trait Implementation<T> {
    fn get_value(&self, container: &Container) -> T;
}

impl<T> Implementation<T> for T
where
    T: 'static + Clone,
{
    fn get_value(&self, _container: &Container) -> T {
        self.clone()
    }
}

impl<T> Implementation<T> for Fn(&Container) -> T {
    fn get_value(&self, container: &Container) -> T {
        (self)(container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn resolves_none_when_not_registered() {
        let container = Container::new();
        let resolved: Option<String> = container.resolve();
        assert!(resolved.is_none())
    }

    #[test]
    fn resolves_string() {
        let mut container = Container::new();
        container.register(String::new());

        let resolved: Option<String> = container.resolve();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_rc_of_trait_object() {
        let mut container = Container::new();
        container.register(Rc::new(FooImpl::new()) as Rc<dyn Foo>);

        let resolved: Option<Rc<dyn Foo>> = container.resolve();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_factory_of_rc_of_trait_object() {
        let mut container = Container::new();
        let factory = Box::new(|_container: &Container| Rc::new(FooImpl::new()) as Rc<dyn Foo>);
        container.register(factory);

        let resolved = container.resolve::<Rc<dyn Foo>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_factory_of_box_of_trait_object() {
        let mut container = Container::new();
        let factory = Box::new(|_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>);
        container.register(factory);

        let resolved: Option<Box<dyn Foo>> = container.resolve();
        assert!(resolved.is_some())
    }

    trait Foo {}

    struct FooImpl {}

    impl FooImpl {
        fn new() -> Self {
            FooImpl {}
        }
    }

    impl Foo for FooImpl {}
}
