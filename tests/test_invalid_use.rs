#[test]
fn invalid_use() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
    since_1_49(&t);
    nightly(&t);
}

#[rustversion::since(1.49)]
fn since_1_49(t: &trybuild::TestCases) {
    t.compile_fail("tests/invalid-use/since-1-49/*.rs");
}

#[rustversion::before(1.49)]
fn since_1_49(_: &trybuild::TestCases) {}

#[rustversion::nightly]
fn nightly(t: &trybuild::TestCases) {
    t.compile_fail("tests/invalid-use/nightly/*.rs");
}

#[rustversion::not(nightly)]
fn nightly(_: &trybuild::TestCases) {}
