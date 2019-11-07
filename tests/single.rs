use trybuild::TestCases;

#[test]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("ui/single/*.rs");
}
