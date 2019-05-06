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
fn test() {
    let mut container = Container::new();
    container.register_autoresolved(|foo: Option<Foo>| foo.unwrap());
}
