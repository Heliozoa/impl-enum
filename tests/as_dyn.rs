struct A;
struct B;
struct C;
struct D;

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
impl T for C {
    fn f(&self) -> &'static str {
        "C"
    }
    fn mut_f(&mut self) -> &'static str {
        "mut C"
    }
}

#[impl_enum::as_dyn(T)]
enum E {
    A { a: A, b: B, c: C, d: D },
    B(B, C, D),
    C(C, D),
}

#[test]
fn call() {
    let a = E::A {
        a: A,
        b: B,
        c: C,
        d: D,
    };
    assert_eq!("A", a.as_dyn_t().f());

    let mut b = E::B(B, C, D);
    assert_eq!("mut B", b.as_dyn_t_mut().mut_f());

    let c = E::C(C, D);
    assert_eq!("mut C", c.into_dyn_t().mut_f());
}
