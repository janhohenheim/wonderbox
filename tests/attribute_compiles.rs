use wonderbox_codegen::resolve_dependencies;

#[derive(Debug, Default)]
struct Foo;

#[resolve_dependencies]
impl Foo {
    fn new() -> Self {
        Foo::default()
    }
}

#[test]
fn test() {
    let _foo = Foo::new();
}
