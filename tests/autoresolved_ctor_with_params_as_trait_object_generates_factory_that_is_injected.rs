use wonderbox::{register, resolve_dependencies, Container};

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

trait Bar {
    fn create_foo(&self) -> Box<dyn Foo>;
}

struct BarImpl {
    foo_factory: Box<dyn Fn() -> Box<dyn Foo>>,
}

#[resolve_dependencies]
impl BarImpl {
    fn new(foo_factory: Box<dyn Fn() -> Box<dyn Foo>>) -> Self {
        Self { foo_factory }
    }
}

impl Bar for BarImpl {
    fn create_foo(&self) -> Box<dyn Foo> {
        (self.foo_factory)()
    }
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register(|_| "foo".to_string());
    register!(container, FooImpl as Box<dyn Foo>);
    register!(container, BarImpl as Box<dyn Bar>);;

    let bar = container.resolve::<Box<dyn Bar>>();
    assert!(bar.is_some());

    let _foo = bar.unwrap().create_foo();
}
