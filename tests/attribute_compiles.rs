use wonderbox::autoresolvable;

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
    let _foo = Foo::new();
}
