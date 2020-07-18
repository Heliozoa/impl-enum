use std::collections::HashSet;

#[impl_enum::with_methods {
    fn len(&self) -> usize;
}]
enum Enum {
    Vec { vec: Vec<u8> },
    Set(HashSet<String>),
}

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
}
