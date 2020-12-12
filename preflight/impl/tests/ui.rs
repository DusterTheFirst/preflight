use trybuild::TestCases;

#[test]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/fail-*.rs");
    t.pass("tests/ui/pass-*.rs");
}