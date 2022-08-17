struct A;
struct B;
struct C;

trait T {
    fn f(&self) -> &'static str;
    fn mut_f(&mut self) -> &'static str;
}

impl T for A {
    fn f(&self) -> &'static str {
        "A"
    }
    fn mut_f(&mut self) -> &'static str {
        "mut A"
    }
}
impl T for B {
    fn f(&self) -> &'static str {
        "B"
    }
    fn mut_f(&mut self) -> &'static str {
        "mut B"
    }
}

#[impl_enum::as_dyn(T)]
enum E {
    A { a: A, b: B, c: C },
    B(B, C),
}

#[test]
fn call() {
    let a = E::A { a: A, b: B, c: C };
    assert_eq!("A", a.as_dyn_t().f());

    let mut b = E::B(B, C);
    assert_eq!("mut B", b.as_dyn_t_mut().mut_f());
}
