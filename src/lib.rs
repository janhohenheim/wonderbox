//! A minimalistic [IoC] library.
//!
//! # Examples
//! ```
//! use wonderbox::{autoresolvable, register_autoresolvable, Container};
//!
//! trait Foo {}
//!
//! #[derive(Debug, Default)]
//! struct FooImpl {
//!     stored_string: String,
//! }
//!
//! #[autoresolvable]
//! impl FooImpl {
//!     fn new(stored_string: String) -> Self {
//!         Self { stored_string }
//!     }
//! }
//!
//! impl Foo for FooImpl {}
//!
//! let mut container = Container::new();
//! container.register(|_| "foo".to_string());
//! register_autoresolvable!(container, FooImpl as Box<dyn Foo>);
//!
//! let foo = container.try_resolve::<Box<dyn Foo>>();
//! assert!(foo.is_some())
//! ```
//!
//! [IoC]: https://en.wikipedia.org/wiki/Inversion_of_control

#![feature(custom_attribute)]
#![feature(core_intrinsics)]
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

pub use wonderbox_codegen::autoresolvable;

use crate::internal::AutoResolvable;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// The IoC container
#[derive(Default, Debug, Clone)]
pub struct Container {
    registered_types: HashMap<&'static str, Arc<dyn Any + Send + Sync>>,
    registered_type_factories: HashMap<&'static str, Arc<dyn Any + Send + Sync>>,
}

type ImplementationFactory<T> = dyn Fn(&Container) -> T + Send + Sync;

impl Container {
    /// Create a new empty [`Container`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a function that returns the implementation of a type.
    /// Can be used to try_resolve dependencies.
    ///
    /// # Examples
    ///
    /// Registering a simple factory:
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    /// container.register(|_| String::new());
    /// ```
    ///
    /// Registering a factory for a trait object with dependencies:
    /// ```
    /// use std::rc::Rc;
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    /// container.register(|_| "I'm a dependency".to_string());
    /// container.register(|container| {
    ///     let dependency = container.try_resolve::<String>().unwrap();
    ///     let registered_type = FooImpl {
    ///         stored_string: dependency,
    ///     };
    ///     Box::new(registered_type) as Box<dyn Foo>
    /// });
    ///
    /// trait Foo {}
    /// struct FooImpl {
    ///     stored_string: String,
    /// }
    /// impl Foo for FooImpl {}
    /// ```
    ///
    /// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
    pub fn register<T>(
        &mut self,
        implementation_factory: impl Fn(&Container) -> T + 'static + Send + Sync + Clone,
    ) -> &mut Self
    where
        T: 'static,
    {
        let registered_implementation_factory: Box<ImplementationFactory<T>> = {
            let implementation_factory = implementation_factory.clone();
            Box::new(move |container| implementation_factory(container))
        };
        self.registered_types.insert(
            type_name::<T>(),
            Arc::new(registered_implementation_factory),
        );

        let partially_applied_implementation_factory: Box<
            ImplementationFactory<Box<dyn Fn() -> T>>,
        > = Box::new(move |container: &Container| {
            let implementation_factory = implementation_factory.clone();
            let container = container.clone();
            Box::new(move || implementation_factory(&container))
        });

        self.registered_type_factories.insert(
            type_name::<Box<dyn Fn() -> T>>(),
            Arc::new(partially_applied_implementation_factory),
        );

        self
    }

    /// Register a type while automatically resolving its dependencies.
    /// Only works with types which have an `#[autoresolvable] attribute on an `Impl` containing constructors.`
    ///
    /// For most registrations it will be easier to use the convenience macro [`register_autoresolvable!`].
    ///
    /// # Examples
    /// ```
    /// use wonderbox::{autoresolvable, register_autoresolvable, Container};
    ///
    /// trait Foo {}
    ///
    /// #[derive(Debug, Default)]
    /// struct FooImpl {
    ///     stored_string: String,
    /// }
    ///
    /// #[autoresolvable]
    /// impl FooImpl {
    ///     fn new(stored_string: String) -> Self {
    ///         Self { stored_string }
    ///     }
    /// }
    ///
    /// impl Foo for FooImpl {}
    ///
    /// let mut container = Container::new();
    /// container.register(|_| "foo".to_string());
    ///
    /// // The following two calls are equivalent
    /// container
    ///     .register_autoresolvable(|foo: Option<FooImpl>| Box::new(foo.unwrap()) as Box<dyn Foo>);
    /// register_autoresolvable!(container, FooImpl as Box<dyn Foo>);
    ///
    /// let foo = container.try_resolve::<Box<dyn Foo>>();
    /// assert!(foo.is_some())
    /// ```
    pub fn register_autoresolvable<ResolvedType, RegisteredType>(
        &mut self,
        registration_fn: impl Fn(Option<ResolvedType>) -> RegisteredType + 'static + Send + Sync + Clone,
    ) -> &mut Self
    where
        ResolvedType: AutoResolvable,
        RegisteredType: 'static,
    {
        self.register(move |container| registration_fn(ResolvedType::try_resolve(container)));
        self
    }

    /// Register all the element from another container into this container.
    /// # Examples
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut first_container = Container::new();
    /// first_container.register(|_| "foo".to_string());
    ///
    /// let mut second_container = Container::new();
    /// second_container.register(|container| {
    ///     let dependency = container.try_resolve::<String>().unwrap();
    ///     let foo = FooImpl {
    ///         stored_string: dependency,
    ///     };
    ///     Box::new(foo) as Box<dyn Foo>
    /// });
    ///
    /// first_container.extend(second_container);
    ///
    /// trait Foo {}
    /// struct FooImpl {
    ///     stored_string: String,
    /// }
    /// impl Foo for FooImpl {}
    /// ```
    pub fn extend(&mut self, container: Container) -> &mut Self {
        self.registered_types
            .extend(container.registered_types.into_iter());
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
    /// container.register(|_| "some dependency".to_string());
    ///
    /// let resolved = container.try_resolve::<String>();
    /// assert!(resolved.is_some())
    /// ```
    ///
    /// Resolve a trait object
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    ///
    /// container.register(|_| "foo".to_string());
    /// container.register(|container| {
    ///     let dependency = container.try_resolve::<String>().unwrap();
    ///     let foo = FooImpl {
    ///         stored_string: dependency,
    ///     };
    ///     Box::new(foo) as Box<dyn Foo>
    /// });
    ///
    /// let resolved = container.try_resolve::<Box<dyn Foo>>();
    /// assert!(resolved.is_some());
    ///
    /// trait Foo {}
    /// struct FooImpl {
    ///     stored_string: String,
    /// }
    /// impl Foo for FooImpl {}
    /// ```
    pub fn try_resolve<T>(&self) -> Option<T>
    where
        T: 'static,
    {
        let key = type_name::<T>();
        let resolvable_type = self
            .registered_types
            .get(key)
            .or_else(|| self.registered_type_factories.get(key))?;
        let implementation_factory = resolvable_type
            .downcast_ref::<Box<ImplementationFactory<T>>>()
            .unwrap_or_else(|| {
                panic!(
                    "Internal error: Couldn't downcast internally stored registered type to resolved \
                     type `{}`.\nYou've encountered a Wonderbox bug. Please consider opening an \
                     issue at https://github.com/jnferner/wonderbox/issues/new\nAdditional info: {}",
                    type_name::<T>(),
                    self.resolution_failure_help()
                )
            });
        let value = implementation_factory(self);
        Some(value)
    }

    /// Retrieves the registered implementation of the specified type.
    /// # Errors
    /// Panics with a nice error message if the type was not registered
    /// # Examples
    /// Resolve a simple registered type
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    /// container.register(|_| "some dependency".to_string());
    ///
    /// let resolved = container.try_resolve::<String>();
    /// assert!(resolved.is_some())
    /// ```
    ///
    /// Resolve a trait object
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    ///
    /// container.register(|_| "foo".to_string());
    /// container.register(|container| {
    ///     let dependency = container.try_resolve::<String>().unwrap();
    ///     let foo = FooImpl {
    ///         stored_string: dependency,
    ///     };
    ///     Box::new(foo) as Box<dyn Foo>
    /// });
    ///
    /// let resolved = container.resolve::<Box<dyn Foo>>();
    ///
    /// trait Foo {}
    /// struct FooImpl {
    ///     stored_string: String,
    /// }
    /// impl Foo for FooImpl {}
    /// ```
    pub fn resolve<T>(&self) -> T
    where
        T: 'static,
    {
        self.try_resolve::<T>().unwrap_or_else(|| {
            panic!(
                "Wonderbox failed to resolve the type `{}`.\nHelp: {}",
                type_name::<T>(),
                self.resolution_failure_help()
            )
        })
    }

    fn resolution_failure_help(&self) -> String {
        if self.registered_types.is_empty() {
            "No registered types were found.".into()
        } else {
            format!(
                "The following registered types were found.\n{}{}",
                self.pretty_print_registered_types(),
                "In addition, the following factories were generated for each type `T`.\n- \
                 std::boxed::Box<dyn Fn() -> T>"
            )
        }
    }

    fn pretty_print_registered_types(&self) -> String {
        // Chosen arbitrarily
        const AVERAGE_LENGTH_OF_TYPE_NAME: usize = 20;

        let initial_capacity = self.registered_types.len() * AVERAGE_LENGTH_OF_TYPE_NAME;
        let mut registered_type_names = self.registered_type_names();
        registered_type_names.sort();

        registered_type_names
            .iter()
            .map(|type_name| format!("- {}\n", type_name))
            .fold(String::with_capacity(initial_capacity), |acc, type_name| {
                acc + &type_name
            })
    }

    fn registered_type_names(&self) -> Vec<String> {
        self.registered_types
            .keys()
            .map(|&key| String::from(key))
            .collect()
    }
}

/// Primary way to register types annotated with `#[autoresolvable]`.
/// This macro is syntax sugar over [`register_autoresolvable`]
///
/// # Examples
/// ```
/// use wonderbox::{autoresolvable, register_autoresolvable, Container};
///
/// trait Foo {}
///
/// #[derive(Debug, Default)]
/// struct FooImpl {
///     stored_string: String,
/// }
///
/// #[autoresolvable]
/// impl FooImpl {
///     fn new(stored_string: String) -> Self {
///         Self { stored_string }
///     }
/// }
///
/// impl Foo for FooImpl {}
///
/// let mut container = Container::new();
/// container.register(|_| "foo".to_string());
/// register_autoresolvable!(container, FooImpl as Box<dyn Foo>);
///
/// let foo = container.try_resolve::<Box<dyn Foo>>();
/// assert!(foo.is_some())
/// ```
#[macro_export]
macro_rules! register_autoresolvable {
    ($container: ident, $implementation: ty) => {
        $container.register_autoresolvable(|implementation: Option<$implementation>| {
            implementation.unwrap()
        })
    };
    ($container: ident, $implementation: ty as $outer_type:tt <$inner_type: ty>) => {
        $container.register_autoresolvable(|implementation: Option<$implementation>| {
            $outer_type::new(implementation.unwrap()) as $outer_type<$inner_type>
        })
    };
}

fn type_name<T>() -> &'static str {
    unsafe { std::intrinsics::type_name::<T>() }
}

#[doc(hidden)]
pub mod internal {
    use super::*;

    pub trait AutoResolvable: Sized {
        fn try_resolve(container: &Container) -> Option<Self>;
    }
}

#[cfg(test)]
#[allow(clippy::blacklisted_name)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn resolves_none_when_not_registered() {
        let container = Container::new();
        let resolved = container.try_resolve::<String>();
        assert!(resolved.is_none())
    }

    #[test]
    #[should_panic(expected = "Wonderbox failed to resolve the type \
                               `std::string::String`.\nHelp: No registered types were found.")]
    fn panics_when_unwraping_type_that_is_not_registered() {
        let container = Container::new();
        let _resolved = container.resolve::<String>();
    }

    #[test]
    #[should_panic(
        expected = "Wonderbox failed to resolve the type `std::boxed::Box<dyn std::ops::Fn() -> \
                    std::boxed::Box<dyn tests::Foo>>`.\nHelp: No registered types were found."
    )]
    fn panics_when_unwraping_trait_object_that_is_not_registered() {
        let container = Container::new();
        let _resolved = container.resolve::<Box<dyn Fn() -> Box<dyn Foo>>>();
    }

    #[test]
    #[should_panic(
        expected = "Wonderbox failed to resolve the type `std::boxed::Box<dyn \
                    tests::Bar>`.\nHelp: The following registered types were found.\n- \
                    std::boxed::Box<dyn tests::Foo>\n- std::string::String\nIn addition, the \
                    following factories were generated for each type `T`.\n- std::boxed::Box<dyn \
                    Fn() -> T>"
    )]
    fn panics_when_unwraping_trait_object_that_is_not_registered_when_other_types_are_registered() {
        let mut container = Container::new();
        container.register(|_| "foo".to_string());
        container.register(|_| Box::new(FooImpl::new()) as Box<dyn Foo>);
        let _resolved = container.resolve::<Box<dyn Bar>>();
    }

    #[test]
    fn resolves_factory_of_rc_of_trait_object() {
        let mut container = Container::new();
        let factory = |_container: &Container| Rc::new(FooImpl::new()) as Rc<dyn Foo>;
        container.register(factory);

        let resolved = container.try_resolve::<Rc<dyn Foo>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_factory_of_box_of_trait_object() {
        let mut container = Container::new();
        let factory = |_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>;
        container.register(factory);

        let resolved = container.try_resolve::<Box<dyn Foo>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_and_unwraps_factory_of_box_of_trait_object() {
        let mut container = Container::new();
        let factory = |_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>;
        container.register(factory);

        let _resolved = container.resolve::<Box<dyn Foo>>();
    }

    #[test]
    fn resolves_boxed_factory_of_box_of_trait_object() {
        let mut container = Container::new();
        let factory = |_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>;
        let boxed_factory = Box::new(factory);
        container.register(boxed_factory);

        let resolved = container.try_resolve::<Box<dyn Foo>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_boxed_factory_with_clone_dependency() {
        let mut container = Container::new();

        container.register(|_| "foo".to_string());
        container.register(|container| {
            let dependency = container.try_resolve::<String>().unwrap();
            let bar = BarImpl::new(dependency);
            Box::new(bar) as Box<dyn Bar>
        });

        let resolved = container.try_resolve::<Box<dyn Bar>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_boxed_factory_with_factory_dependency() {
        let mut container = Container::new();

        container.register(|_| "foo".to_string());
        container.register(|_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>);
        container.register(|container| {
            let clone_dependency = container.try_resolve::<String>().unwrap();
            let _factory_dependency = container.try_resolve::<Box<dyn Foo>>().unwrap();

            let bar = BarImpl::new(clone_dependency);
            Box::new(bar) as Box<dyn Bar>
        });

        let resolved = container.try_resolve::<Box<dyn Bar>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn generates_factory_from_type_with_dependency() {
        let mut container = Container::new();

        container.register(|_| "foo".to_string());
        container.register(|_container: &Container| Box::new(FooImpl::new()) as Box<dyn Foo>);
        container.register(|container| {
            let clone_dependency = container.try_resolve::<String>().unwrap();
            let _factory_dependency = container.try_resolve::<Box<dyn Foo>>().unwrap();

            let bar = BarImpl::new(clone_dependency);
            Box::new(bar) as Box<dyn Bar>
        });

        let resolved = container.try_resolve::<Box<dyn Fn() -> Box<dyn Bar>>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_type_from_merged_containers() {
        let mut first_container = Container::new();
        first_container.register(|_| "foo".to_string());

        let mut second_container = Container::new();
        second_container.register(|container| {
            let dependency = container.try_resolve::<String>().unwrap();
            let bar = BarImpl::new(dependency);
            Box::new(bar) as Box<dyn Bar>
        });

        first_container.extend(second_container);

        let resolved = first_container.try_resolve::<Box<dyn Bar>>();
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
