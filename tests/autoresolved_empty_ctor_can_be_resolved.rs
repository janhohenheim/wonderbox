use wonderbox::{resolve_dependencies, Container};

#[derive(Debug, Default)]
struct Foo;

#[resolve_dependencies]
impl Foo {
    fn new() -> Self {
        Foo::default()
    }
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register_autoresolved(Option::<Foo>::unwrap);

    let foo = container.resolve::<Foo>();
    assert!(foo.is_some());
}
