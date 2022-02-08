#[test]
#[cfg(not(miri))]
#[cfg(feature = "stable_deref_trait")]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/circular.rs");
    t.compile_fail("tests/compile_fail/unstable_deref.rs");
}
