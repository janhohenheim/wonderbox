use wonderbox::{autoresolvable, register_autoresolvable, Container};

#[derive(Debug, Default)]
struct Foo<T> {
    stored_generic: T,
}

#[autoresolvable]
impl<T> Foo<T>
where
    T: 'static,
{
    fn new(stored_generic: T) -> Self {
        Self { stored_generic }
    }
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test() {
    let mut container = Container::new();
    container.register(|_| "foo".to_string());
    container.register(|_| 123u32);
    register_autoresolvable!(container, Foo<String>);
    register_autoresolvable!(container, Foo<u32>);

    let _string = container.resolve::<Foo<String>>();
    let _u32 = container.resolve::<Foo<u32>>();
}
