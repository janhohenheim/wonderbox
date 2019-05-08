use wonderbox::resolve_dependencies;

#[derive(Debug, Default)]
struct Foo;

#[resolve_dependencies]
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
