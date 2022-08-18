#![cfg(feature = "with_methods")]

use std::collections::HashSet;

#[impl_enum::with_methods {
    fn len(&self) -> usize
    fn associated_fn(s: &str)
}]
enum Enum {
    Vec { vec: Vec<u8> },
    Set(HashSet<String>),
}

trait TestTrait {
    fn associated_fn(s: &str) {
        println!("{}", s);
    }
}

impl<T> TestTrait for Vec<T> {}

impl<T> TestTrait for HashSet<T> {}

#[test]
fn test() {
    let e = Enum::Vec {
        vec: vec![1, 2, 3, 4],
    };
    assert_eq!(e.len(), 4);

    let mut set = HashSet::new();
    set.insert("abcd".to_string());
    set.insert("bcde".to_string());
    let e = Enum::Set(set);
    assert_eq!(e.len(), 2);

    e.associated_fn("hello!");
}
