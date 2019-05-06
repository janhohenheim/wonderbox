use wonderbox::Container;
use wonderbox_codegen::resolve_dependencies;

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
fn test() {
    let mut container = Container::new();
    container.register_autoresolved(|foo: Option<Foo>| foo.unwrap());

    let foo_factory = container.resolve::<Box<dyn Fn() -> Foo>>();
    assert!(foo_factory.is_some());
}
