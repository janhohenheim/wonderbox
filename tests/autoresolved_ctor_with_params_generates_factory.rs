use wonderbox::{resolve_dependencies, Container};

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
    container.register_autoresolvable(Option::<Foo>::unwrap);

    let foo_factory = container.resolve::<Box<dyn Fn() -> Foo>>();
    assert!(foo_factory.is_some());
}
