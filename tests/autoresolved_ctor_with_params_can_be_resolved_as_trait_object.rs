use wonderbox::{autoresolvable, Container};

trait Foo {}

#[derive(Debug, Default)]
struct FooImpl {
    stored_string: String,
}

#[autoresolvable]
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
    container.register(|_| "foo".to_string());
    container
        .register_autoresolvable(|foo: Option<FooImpl>| Box::new(foo.unwrap()) as Box<dyn Foo>);

    let foo = container.try_resolve::<Box<dyn Foo>>();
    assert!(foo.is_some())
}
