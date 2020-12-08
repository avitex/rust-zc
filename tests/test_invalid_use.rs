#[rustversion::since(1.49)]
#[test]
fn invalid_use_since_1_49() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
    t.compile_fail("tests/invalid-use/since-1-49.rs");
}

#[rustversion::before(1.49)]
#[test]
fn invalid_use_before_1_49() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
    t.compile_fail("tests/invalid-use/before-1-49/*.rs");
}
