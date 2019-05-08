use wonderbox::{register, resolve_dependencies, Container};

#[derive(Debug, Default)]
struct Foo {
    stored_string: String,
}

#[resolve_dependencies]
impl Foo {
    fn new(stored_string: String) -> Self {
        Self { stored_string }
    }
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register_clone("foo".to_string());
    register!(container, Foo);

    let foo = container.resolve::<Foo>();
    assert!(foo.is_some())
}
