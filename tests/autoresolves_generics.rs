use wonderbox::{autoresolvable, register_autoresolvable, Container};

trait Foo {}

#[derive(Debug, Default)]
struct FooImpl<T> {
    stored_generic: T,
}

#[autoresolvable]
impl<T> FooImpl<T> {
    fn new(stored_generic: T) -> Self {
        Self { stored_generic }
    }
}

impl<T> Foo for FooImpl<T> {}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register(|_| "foo".to_string());
    register_autoresolvable!(container, FooImpl as Box<dyn Foo>);

    let foo = container.try_resolve::<Box<dyn Foo>>();
    assert!(foo.is_some())
}
