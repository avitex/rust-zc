#[rustversion::stable]
#[test]
fn invalid_use() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
}
