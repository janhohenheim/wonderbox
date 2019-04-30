#![feature(specialization)]

use core::any::TypeId;
use std::any::Any;
use std::collections::HashMap;

#[derive(Default)]
struct Container {
    registered_types: HashMap<TypeId, Implementation>,
}

impl Container {
    fn new() -> Self {
        Self::default()
    }

    fn register<T>(&mut self, implementation: T) -> &mut Self
    where
        T: 'static,
    {
        let implementation = Implementation::Concrete(Box::new(implementation));
        self.registered_types
            .insert(TypeId::of::<T>(), implementation);
        self
    }

    fn register_factory<T>(&mut self, factory: Box<Fn(&Container) -> T>) -> &mut Self
    where
        T: 'static,
    {
        let implementation = Implementation::Factory(Box::new(factory));
        self.registered_types
            .insert(TypeId::of::<T>(), implementation);
        self
    }
}

trait Resolvable<T> {
    fn resolve(&self) -> Option<T>;
}

impl<T> Resolvable<T> for Container
where
    T: 'static,
{
    default fn resolve(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let resolvable_type = self.registered_types.get(&type_id)?;
        let implementation = match resolvable_type {
            Implementation::Factory(factory) => {
                let factory = factory
                    .downcast_ref::<Box<dyn Fn(&Container) -> T>>()
                    .expect("Internal error: Couldn't downcast stored type to resolved type");
                factory(&self)
            }
            _ => panic!(),
        };

        Some(implementation)
    }
}

impl<T> Resolvable<T> for Container
where
    T: 'static + Clone,
{
    fn resolve(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let resolvable_type = self.registered_types.get(&type_id)?;
        let implementation = match resolvable_type {
            Implementation::Concrete(implementation) => implementation
                .downcast_ref::<T>()
                .expect("Internal error: Couldn't downcast stored type to resolved type")
                .clone(),
            Implementation::Factory(factory) => {
                let factory = factory
                    .downcast_ref::<Box<dyn Fn(&Container) -> T>>()
                    .expect("Internal error: Couldn't downcast stored type to resolved type");
                factory(&self)
            }
        };

        Some(implementation)
    }
}

enum Implementation {
    Concrete(Box<dyn Any>),
    Factory(Box<dyn Any>),
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
        container.register_factory(factory);

        let resolved: Option<Rc<dyn Foo>> = container.resolve();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_factory_of_box_of_trait_object() {
        let mut container = Container::new();
        let factory = Box::new(|_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>);
        container.register_factory(factory);

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
