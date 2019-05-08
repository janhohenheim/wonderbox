use wonderbox::{resolve_dependencies, Container};

trait Foo {}

#[derive(Debug, Default)]
struct FooImpl {
    stored_string: String,
}

#[resolve_dependencies]
impl FooImpl {
    fn new(stored_string: String) -> Self {
        Self { stored_string }
    }
}

impl Foo for FooImpl {}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register_factory(|_| "foo".to_string());
    container.register_autoresolved(|foo: Option<FooImpl>| Box::new(foo.unwrap()) as Box<dyn Foo>);

    let foo = container.resolve::<Box<dyn Foo>>();
    assert!(foo.is_some())
}
