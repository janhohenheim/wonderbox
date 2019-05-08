# Wonderbox

[![Build Status](https://travis-ci.com/jnferner/wonderbox.svg?branch=master)](https://travis-ci.com/jnferner/wonderbox)
[![Latest Version](https://img.shields.io/crates/v/wonderbox.svg)](https://crates.io/crates/wonderbox)
[![Documentation](https://docs.rs/wonderbox/badge.svg)](https://docs.rs/wonderbox)


A minimalistic [IoC](https://en.wikipedia.org/wiki/Inversion_of_control) library.

## Examples

```rust
use wonderbox::{register, Container, resolve_dependencies};

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

#[test]
fn test() {
    let mut container = Container::new();
    container.register(|_| "foo".to_string());
    register!(container, FooImpl as Box<dyn Foo>);

    let foo = container.resolve::<Box<dyn Foo>>();
    assert!(foo.is_some())
}

```
