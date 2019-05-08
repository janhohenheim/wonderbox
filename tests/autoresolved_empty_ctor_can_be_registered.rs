use wonderbox::{autoresolvable, Container};

#[derive(Debug, Default)]
struct Foo;

#[autoresolvable]
impl Foo {
    fn new() -> Self {
        Foo::default()
    }
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register_autoresolvable(Option::<Foo>::unwrap);
}
