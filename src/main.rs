use core::any::TypeId;
use std::any::Any;
use std::collections::HashMap;

#[derive(Default)]
struct Container {
    shared_items: HashMap<TypeId, Box<dyn Any>>,
}

impl Container {
    fn new() -> Self {
        Self::default()
    }

    fn register<T>(&mut self, implementation: T) -> &mut Self
    where
        T: 'static,
    {
        self.shared_items
            .insert(TypeId::of::<T>(), Box::new(implementation));
        self
    }

    fn resolve<T>(&self) -> Option<T>
    where
        T: 'static + Clone,
    {
        let type_id = TypeId::of::<T>();
        let resolvable_type = self.shared_items.get(&type_id)?;
        Some(
            resolvable_type
                .downcast_ref::<T>()
                .expect("Internal error: Couldn't downcast stored type to resolved type")
                .clone(),
        )
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
        container.register(String::new());

        let resolved = container.resolve::<String>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_rc_of_trait_object() {
        let mut container = Container::new();
        container.register(Rc::new(FooImpl::new()) as Rc<dyn Foo>);

        let resolved = container.resolve::<Rc<dyn Foo>>();
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
