#![feature(specialization)]

use core::any::TypeId;
use std::any::Any;
use std::collections::HashMap;

#[derive(Default)]
struct Container {
    registered_types: HashMap<TypeId, Box<dyn Any>>,
}

type ImplementationFactory<T> = dyn Fn(&Container) -> T;

impl Container {
    fn new() -> Self {
        Self::default()
    }

    fn register_clone<T>(&mut self, implementation: T) -> &mut Self
    where
        T: 'static + Clone,
    {
        let implementation_factory: Box<ImplementationFactory<T>> =
            Box::new(move |_container: &Container| implementation.clone());
        self.registered_types
            .insert(TypeId::of::<T>(), Box::new(implementation_factory));
        self
    }

    fn register_factory<Factory, Implementation>(
        &mut self,
        implementation_factory: Factory,
    ) -> &mut Self
    where
        Factory: 'static + Fn(&Container) -> Implementation,
        Implementation: 'static,
    {
        let implementation_factory: Box<ImplementationFactory<Implementation>> =
            Box::new(implementation_factory);
        self.registered_types.insert(
            TypeId::of::<Implementation>(),
            Box::new(implementation_factory),
        );
        self
    }

    fn resolve<T>(&self) -> Option<T>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let resolvable_type = self.registered_types.get(&type_id)?;
        let implementation_factory = resolvable_type
            .downcast_ref::<Box<ImplementationFactory<T>>>()
            .expect("Internal error: Couldn't downcast stored type to resolved type");
        let value: T = implementation_factory(&self);
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn resolves_none_when_not_registered() {
        let container = Container::new();
        let resolved = container.resolve::<String>();
        assert!(resolved.is_none())
    }

    #[test]
    fn resolves_string() {
        let mut container = Container::new();
        container.register_clone(String::new());

        let resolved = container.resolve::<String>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_rc_of_trait_object() {
        let mut container = Container::new();
        container.register_clone(Rc::new(FooImpl::new()) as Rc<dyn Foo>);

        let resolved = container.resolve::<Rc<dyn Foo>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_factory_of_rc_of_trait_object() {
        let mut container = Container::new();
        let factory = |_container: &Container| Rc::new(FooImpl::new()) as Rc<dyn Foo>;
        container.register_factory(factory);

        let resolved = container.resolve::<Rc<dyn Foo>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_factory_of_box_of_trait_object() {
        let mut container = Container::new();
        let factory = |_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>;
        container.register_factory(factory);

        let resolved = container.resolve::<Box<dyn Foo>>();
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
