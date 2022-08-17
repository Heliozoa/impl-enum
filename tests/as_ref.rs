struct A(String);
struct B(&'static str);
struct C<'a>(&'a str);

impl AsRef<str> for A {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for B {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl AsRef<str> for C<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[impl_enum::as_ref(str)]
enum E<'a> {
    A { a: A, b: B, c: C<'a> },
    B(B, C<'a>),
}

#[test]
fn call() {
    let c = String::from("c");
    let a = E::A {
        a: A("a".to_string()),
        b: B("b"),
        c: C(&c),
    };
    assert_eq!("a", a.as_ref_str());
}
