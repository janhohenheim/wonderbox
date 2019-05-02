//! A minimalistic [IoC] library.
//!
//! [IoC]: https://en.wikipedia.org/wiki/Inversion_of_control

#![warn(missing_docs, clippy::dbg_macro, clippy::unimplemented)]
#![deny(
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    clippy::default_trait_access,
    clippy::enum_glob_use,
    clippy::needless_borrow,
    clippy::large_digit_groups,
    clippy::explicit_into_iter_loop
)]

use core::any::TypeId;
use std::any::Any;
use std::collections::HashMap;

/// The IoC container
#[derive(Default, Debug)]
pub struct Container {
    registered_types: HashMap<TypeId, Box<dyn Any>>,
}

type ImplementationFactory<T> = dyn Fn(&Container) -> T;

impl Container {
    /// Create a new empty [`Container`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the implementation of a type that implements [`Clone`].
    ///
    /// # Examples
    ///
    /// Registering a simple type:
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    /// container.register_clone(String::new());
    /// ```
    ///
    /// Registering an Rc of a trait object type:
    /// ```
    /// use wonderbox::Container;
    /// use std::rc::Rc;
    ///
    /// let mut container = Container::new();
    /// container.register_clone(Rc::new(FooImpl)as Rc<dyn Foo>);
    ///
    /// trait Foo {}
    /// struct FooImpl;
    /// impl Foo for FooImpl {}
    /// ```
    ///
    /// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
    pub fn register_clone<T>(&mut self, implementation: T) -> &mut Self
    where
        T: 'static + Clone,
    {
        let implementation_factory: Box<ImplementationFactory<T>> =
            Box::new(move |_container: &Container| implementation.clone());
        self.registered_types
            .insert(TypeId::of::<T>(), Box::new(implementation_factory));
        self
    }

    /// Register a function that returns the implementation of a type.
    /// Can be used to resolve dependencies.
    ///
    /// # Examples
    ///
    /// Registering a simple factory:
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    /// container.register_factory(|_| String::new());
    /// ```
    ///
    /// Registering a factory for a trait object with dependencies:
    /// ```
    /// use wonderbox::Container;
    /// use std::rc::Rc;
    ///
    /// let mut container = Container::new();
    /// let dependency = "I'm a dependency".to_string();
    /// container.register_clone(dependency);
    /// container.register_factory(|container| {
    ///     let dependency = container.resolve::<String>().unwrap();
    ///     let registered_type = FooImpl { stored_string: dependency };
    ///     Box::new(registered_type) as Box<dyn Foo>
    /// });
    ///
    /// trait Foo {}
    /// struct FooImpl {
    ///     stored_string: String
    /// }
    /// impl Foo for FooImpl {}
    /// ```
    ///
    /// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
    pub fn register_factory<Factory, Implementation>(
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

    /// Retrieves the registered implementation of the specified type.
    /// # Errors
    /// Returns `None` if the type was not registered
    /// # Examples
    /// Resolve a simple registered type
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    /// container.register_clone(String::new());
    ///
    /// let resolved = container.resolve::<String>();
    /// assert!(resolved.is_some())
    /// ```
    ///
    /// Resolve a trait object
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    ///
    /// container.register_clone("foo".to_string());
    /// container.register_factory(|container| {
    ///     let dependency = container.resolve::<String>().unwrap();
    ///     let foo = FooImpl { stored_string: dependency };
    ///     Box::new(foo) as Box<dyn Foo>
    /// });
    ///
    /// let resolved = container.resolve::<Box<dyn Foo>>();
    /// assert!(resolved.is_some());
    ///
    /// trait Foo {}
    /// struct FooImpl {
    ///     stored_string: String
    /// }
    /// impl Foo for FooImpl {}
    /// ```
    pub fn resolve<T>(&self) -> Option<T>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let resolvable_type = self.registered_types.get(&type_id)?;
        let implementation_factory = resolvable_type
            .downcast_ref::<Box<ImplementationFactory<T>>>()
            .expect("Internal error: Couldn't downcast stored type to resolved type");
        let value: T = implementation_factory(self);
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

    #[test]
    fn resolves_boxed_factory_of_box_of_trait_object() {
        let mut container = Container::new();
        let factory = |_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>;
        let boxed_factory = Box::new(factory);
        container.register_factory(boxed_factory);

        let resolved = container.resolve::<Box<dyn Foo>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_boxed_factory_with_clone_dependency() {
        let mut container = Container::new();

        container.register_clone("foo".to_string());
        container.register_factory(|container| {
            let dependency = container.resolve::<String>().unwrap();
            let bar = BarImpl::new(dependency);
            Box::new(bar) as Box<dyn Bar>
        });

        let resolved = container.resolve::<Box<dyn Bar>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_boxed_factory_with_factory_dependency() {
        let mut container = Container::new();

        container.register_clone("foo".to_string());
        container
            .register_factory(|_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>);
        container.register_factory(|container| {
            let clone_dependency = container.resolve::<String>().unwrap();
            let _factory_dependency = container.resolve::<Box<dyn Foo>>().unwrap();

            let bar = BarImpl::new(clone_dependency);
            Box::new(bar) as Box<dyn Bar>
        });

        let resolved = container.resolve::<Box<dyn Bar>>();
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

    trait Bar {}

    struct BarImpl {
        _stored_string: String,
    }

    impl BarImpl {
        fn new(stored_string: String) -> Self {
            BarImpl {
                _stored_string: stored_string,
            }
        }
    }

    impl Bar for BarImpl {}

}