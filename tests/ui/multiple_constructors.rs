use wonderbox::autoresolvable;

struct List {
    capacity: usize,
}

#[autoresolvable]
impl List {
    fn new() -> Self {
        List { capacity: 0 }
    }

    fn with_capacity(capacity: usize) -> Self {
        List { capacity }
    }
}

fn main() {}
