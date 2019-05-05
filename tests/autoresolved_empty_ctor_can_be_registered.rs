use wonderbox::Container;
use wonderbox_codegen::resolve_dependencies;

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
    container.register_autoresolved(|foo: Option<Foo>| Some(foo?));
}
