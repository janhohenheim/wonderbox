use wonderbox::{autoresolvable, Container};

trait Foo {}

#[derive(Debug, Default)]
struct FooImpl;

#[autoresolvable]
impl FooImpl {
    fn new() -> Self {
        FooImpl::default()
    }
}

impl Foo for FooImpl {}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container
        .register_autoresolvable(|foo: Option<FooImpl>| Box::new(foo.unwrap()) as Box<dyn Foo>);

    let foo = container.try_resolve::<Box<dyn Foo>>();
    assert!(foo.is_some());
}
