use wonderbox::{autoresolvable, register_autoresolvable, Container};

#[derive(Debug, Default)]
struct Foo {
    stored_string: String,
}

#[autoresolvable]
impl Foo {
    fn new(stored_string: String) -> Self {
        Self { stored_string }
    }
}

#[test]
fn test() {
    let mut container = Container::new();
    register_autoresolvable!(container, Foo);
}
