#[test]
#[cfg(not(miri))]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/circular.rs");
    t.compile_fail("tests/compile_fail/unstable_deref.rs");
    t.compile_fail("tests/compile_fail/unstable_deref_pinned.rs");
}
