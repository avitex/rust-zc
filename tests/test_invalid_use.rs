#[test]
fn invalid_use() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid-use/*.rs");
    stable(&t);
    since_1_49(&t);
    nightly(&t);
}

#[rustversion::stable]
fn stable(t: &trybuild::TestCases) {
    t.compile_fail("tests/invalid-use/stable/*.rs");
}

#[rustversion::not(stable)]
fn stable(_: &trybuild::TestCases) {}

#[rustversion::all(since(1.49), not(nightly))]
fn since_1_49(t: &trybuild::TestCases) {
    t.compile_fail("tests/invalid-use/since-1-49/*.rs");
}

#[rustversion::any(before(1.49), nightly)]
fn since_1_49(_: &trybuild::TestCases) {}

#[rustversion::nightly]
fn nightly(t: &trybuild::TestCases) {
    t.compile_fail("tests/invalid-use/nightly/*.rs");
}

#[rustversion::not(nightly)]
fn nightly(_: &trybuild::TestCases) {}
