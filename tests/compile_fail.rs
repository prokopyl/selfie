#[test]
#[cfg(not(miri))]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/circular.rs");
    t.compile_fail("tests/compile_fail/clone.rs");
    t.compile_fail("tests/compile_fail/with_referential_mut_from_outer.rs");
}
