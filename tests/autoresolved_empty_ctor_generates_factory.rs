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
fn test() {
    let mut container = Container::new();
    container.register_autoresolved(|foo: Option<Foo>| foo.unwrap());

    let foo_factory = container.resolve::<Box<dyn Fn() -> Foo>>();
    assert!(foo_factory.is_some());
}
