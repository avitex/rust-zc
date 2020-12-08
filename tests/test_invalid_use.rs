#[rustversion::not(nightly)]
#[test]
fn invalid_use_stable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
    t.compile_fail("tests/invalid-use/stable/*.rs");
}

#[rustversion::nightly]
#[test]
fn invalid_use_nightly() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
    t.compile_fail("tests/invalid-use/nightly/*.rs");
}
