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

struct Bar {
    foo_factory: Box<dyn Fn(String) -> Foo>,
}

#[resolve_dependencies]
impl Bar {
    fn new(foo_factory: Box<dyn Fn(String) -> Foo>) -> Self {
        Self { foo_factory }
    }
}

#[test]
fn test() {
    let mut container = Container::new();
    container.register_clone("foo".to_string());
    container.register_autoresolved(|foo: Option<Foo>| foo.unwrap());
    container.register_autoresolved(|bar: Option<Bar>| bar.unwrap());

    let bar = container.resolve::<Bar>();
    assert!(bar.is_some());

    let foo_factory = bar.unwrap().foo_factory;
    let _foo = foo_factory("bar".to_string());
}
