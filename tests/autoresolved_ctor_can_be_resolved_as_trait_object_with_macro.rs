use wonderbox::{autoresolvable, register_autoresolvable, Container};

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
    register_autoresolvable!(container, FooImpl as Box<dyn Foo>);

    let foo = container.resolve::<Box<dyn Foo>>();
    assert!(foo.is_some())
}
