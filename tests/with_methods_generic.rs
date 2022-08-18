#![cfg(feature = "with_methods")]

use impl_enum::with_methods;

trait A {
    fn f() -> &'static str;
}

impl A for () {
    fn f() -> &'static str {
        "A"
    }
}

trait B {
    fn f() -> &'static str;
}

impl B for () {
    fn f() -> &'static str {
        "B"
    }
}

#[with_methods {
    fn f() -> &'static str
}]
enum Generic<T, U>
where
    T: A,
    U: B,
{
    T(T),
    U(U),
}

#[test]
fn t() {
    let generic = Generic::<(), ()>::T(());
    assert_eq!("A", generic.f());
    let generic = Generic::<(), ()>::U(());
    assert_eq!("B", generic.f());
}
