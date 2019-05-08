//! A minimalistic [IoC] library.
//!
//! # Examples
//! ```
//! use wonderbox::{register, Container, resolve_dependencies};
//!
//! trait Foo {}
//!
//! #[derive(Debug, Default)]
//! struct FooImpl {
//!     stored_string: String,
//! }
//!
//! #[resolve_dependencies]
//! impl FooImpl {
//!     fn new(stored_string: String) -> Self {
//!         Self { stored_string }
//!     }
//! }
//!
//! impl Foo for FooImpl {}
//!
//!
//! let mut container = Container::new();
//! container.register_clone("foo".to_string());
//! register!(container, FooImpl as Box<dyn Foo>);
//!
//! let foo = container.resolve::<Box<dyn Foo>>();
//! assert!(foo.is_some())
//!
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

pub use wonderbox_codegen::resolve_dependencies;

use crate::internal::AutoResolvable;
use core::any::TypeId;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// The IoC container
#[derive(Default, Debug, Clone)]
pub struct Container {
    registered_types: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

type ImplementationFactory<T> = dyn Fn(&Container) -> T + Send + Sync;

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
        T: 'static + Send + Sync + Clone,
    {
        self.register_factory(move |_| implementation.clone());

        self
    }

    /// Register a type by its [`Default`] implementation.
    ///
    /// # Examples
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut container = Container::new();
    /// container.register_default::<String>();
    ///
    /// let resolved = container.resolve::<String>();
    /// assert!(resolved.is_some())
    /// ```
    /// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
    pub fn register_default<T>(&mut self) -> &mut Self
    where
        T: 'static + Default,
    {
        self.register_factory(|_| T::default());
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
    pub fn register_factory<T>(
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
            TypeId::of::<T>(),
            Arc::new(registered_implementation_factory),
        );

        let partially_applied_implementation_factory: Box<
            ImplementationFactory<Box<dyn Fn() -> T>>,
        > = Box::new(move |container: &Container| {
            let implementation_factory = implementation_factory.clone();
            let container = container.clone();
            Box::new(move || implementation_factory(&container))
        });

        self.registered_types.insert(
            TypeId::of::<Box<dyn Fn() -> T>>(),
            Arc::new(partially_applied_implementation_factory),
        );

        self
    }

    /// Register a type while automatically resolving its dependencies.
    /// Only works with types which have an `#[resolve_dependencies] attribute on an `Impl` containing constructors.`
    ///
    /// For most registrations it will be easier to use the convenience macro [`register!`].
    ///
    /// # Examples
    /// ```
    /// use wonderbox::{Container, resolve_dependencies};
    ///
    /// trait Foo {}
    ///
    /// #[derive(Debug, Default)]
    /// struct FooImpl {
    ///     stored_string: String,
    /// }
    ///
    /// #[resolve_dependencies]
    /// impl FooImpl {
    ///     fn new(stored_string: String) -> Self {
    ///         Self { stored_string }
    ///     }
    /// }
    ///
    /// impl Foo for FooImpl {}
    ///
    /// let mut container = Container::new();
    /// container.register_clone("foo".to_string());
    /// container.register_autoresolved(|foo: Option<FooImpl>| Box::new(foo.unwrap()) as Box<dyn Foo>);
    ///
    /// let foo = container.resolve::<Box<dyn Foo>>();
    /// assert!(foo.is_some())
    /// ```
    pub fn register_autoresolved<ResolvedType, RegisteredType>(
        &mut self,
        registration_fn: impl Fn(Option<ResolvedType>) -> RegisteredType + 'static + Send + Sync,
    ) -> &mut Self
    where
        ResolvedType: AutoResolvable,
        RegisteredType: 'static,
    {
        let registration_fn = Arc::new(registration_fn);
        let implementation_factory: Box<ImplementationFactory<RegisteredType>> = {
            let registration_fn = registration_fn.clone();
            Box::new(move |container| registration_fn(ResolvedType::resolve(container)))
        };
        self.registered_types.insert(
            TypeId::of::<RegisteredType>(),
            Arc::new(implementation_factory),
        );

        let partially_applied_implementation_factory =
            partially_apply_implementation_factory(registration_fn);

        self.registered_types.insert(
            TypeId::of::<Box<dyn Fn() -> RegisteredType>>(),
            Arc::new(partially_applied_implementation_factory),
        );

        self
    }

    /// Register all the element from another container into this container.
    /// # Examples
    /// ```
    /// use wonderbox::Container;
    ///
    /// let mut first_container = Container::new();
    /// first_container.register_clone("foo".to_string());
    ///
    /// let mut second_container = Container::new();
    /// second_container.register_factory(|container| {
    ///     let dependency = container.resolve::<String>().unwrap();
    ///     let foo = FooImpl { stored_string: dependency };
    ///     Box::new(foo) as Box<dyn Foo>
    /// });
    ///
    /// first_container.register_container(second_container);
    ///
    /// trait Foo {}
    /// struct FooImpl {
    ///     stored_string: String
    /// }
    /// impl Foo for FooImpl {}
    /// ```
    pub fn register_container(&mut self, container: Container) -> &mut Self {
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
            .unwrap_or_else(|| {
                panic!(
                    "Internal error: Couldn't downcast stored \
                     implementation factory to resolved type \"{}\"",
                    type_name::<T>()
                )
            });
        let value: T = implementation_factory(self);
        Some(value)
    }
}

/// Primary way to register types annotated with `#[resolve_dependencies]`.
/// This macro is syntax sugar over [`register_autoresolved`]
///
/// # Examples
/// ```
/// use wonderbox::{register, Container, resolve_dependencies};
///
/// trait Foo {}
///
/// #[derive(Debug, Default)]
/// struct FooImpl {
///     stored_string: String,
/// }
///
/// #[resolve_dependencies]
/// impl FooImpl {
///     fn new(stored_string: String) -> Self {
///         Self { stored_string }
///     }
/// }
///
/// impl Foo for FooImpl {}
///
///
/// let mut container = Container::new();
/// container.register_clone("foo".to_string());
/// register!(container, FooImpl as Box<dyn Foo>);
///
/// let foo = container.resolve::<Box<dyn Foo>>();
/// assert!(foo.is_some())
/// ```
#[macro_export]
macro_rules! register {
    ($container: ident, $implementation: ty) => {
        $container.register_autoresolved(|implementation: Option<$implementation>| {
            implementation.unwrap()
        })
    };
    ($container: ident, $implementation: ty as Box<$registration: ty>) => {
        $container.register_autoresolved(|implementation: Option<$implementation>| {
            Box::new(implementation.unwrap()) as Box<$registration>
        })
    };
}

fn partially_apply_implementation_factory<ResolvedType, RegisteredType>(
    registration_fn: Arc<impl Fn(Option<ResolvedType>) -> RegisteredType + 'static + Send + Sync>,
) -> Box<ImplementationFactory<Box<dyn Fn() -> RegisteredType + Send + Sync>>>
where
    ResolvedType: AutoResolvable,
    RegisteredType: 'static,
{
    Box::new(move |container| {
        let registration_fn = registration_fn.clone();
        let container = container.clone();
        Box::new(move || registration_fn(ResolvedType::resolve(&container)))
    })
}

fn type_name<T>() -> &'static str {
    unsafe { std::intrinsics::type_name::<T>() }
}

#[doc(hidden)]
pub mod internal {
    use super::*;

    pub trait AutoResolvable: Sized {
        fn resolve(container: &Container) -> Option<Self>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::sync::Mutex;

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
    fn resolves_default() {
        let mut container = Container::new();
        container.register_default::<String>();

        let resolved = container.resolve::<String>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_factory_generated_from_default() {
        let mut container = Container::new();
        container.register_default::<String>();

        let resolved = container.resolve::<Box<dyn Fn() -> String>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_rc_of_trait_object() {
        let mut container = Container::new();
        container
            .register_clone(Arc::new(Mutex::new(FooImpl::new())) as Arc<Mutex<dyn Foo + Send>>);

        let resolved = container.resolve::<Arc<Mutex<dyn Foo + Send>>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn generates_factory_of_rc_of_trait_object() {
        let mut container = Container::new();
        container
            .register_clone(Arc::new(Mutex::new(FooImpl::new())) as Arc<Mutex<dyn Foo + Send>>);

        let resolved = container.resolve::<Box<dyn Fn() -> Arc<Mutex<dyn Foo + Send>>>>();
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

    #[test]
    fn generates_factory_from_type_with_dependency() {
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

        let resolved = container.resolve::<Box<dyn Fn() -> Box<dyn Bar>>>();
        assert!(resolved.is_some())
    }

    #[test]
    fn resolves_type_from_merged_containers() {
        let mut first_container = Container::new();
        first_container.register_clone("foo".to_string());

        let mut second_container = Container::new();
        second_container.register_factory(|container| {
            let dependency = container.resolve::<String>().unwrap();
            let bar = BarImpl::new(dependency);
            Box::new(bar) as Box<dyn Bar>
        });

        first_container.register_container(second_container);

        let resolved = first_container.resolve::<Box<dyn Bar>>();
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
