use wonderbox::{autoresolvable, register_autoresolvable, Container};

#[derive(Debug, Default)]
struct Foo {
    stored_string: String,
}

#[autoresolvable]
impl Foo {
    fn new(stored_string: String) -> Self {
        Self { stored_string }
    }
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register(|_| "foo".to_string());
    register_autoresolvable!(container, Foo);

    let foo = container.try_resolve::<Foo>();
    assert!(foo.is_some())
}
