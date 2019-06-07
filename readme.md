# Wonderbox

[![Build Status](https://travis-ci.com/jnferner/wonderbox.svg?branch=master)](https://travis-ci.com/jnferner/wonderbox)
[![Latest Version](https://img.shields.io/crates/v/wonderbox.svg)](https://crates.io/crates/wonderbox)
[![Documentation](https://docs.rs/wonderbox/badge.svg)](https://docs.rs/wonderbox)


A minimalistic [IoC](https://en.wikipedia.org/wiki/Inversion_of_control) library.

## Examples

```rust
use wonderbox::Container;

trait Foo {}

#[derive(Debug, Default)]
struct FooImpl {
    stored_string: String,
}

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
    container.register(|container| Box::new(FooImpl::new(container.resolve())) as Box<dyn Foo>);
    
    let foo = container.resolve::<Box<dyn Foo>>();
}

```
