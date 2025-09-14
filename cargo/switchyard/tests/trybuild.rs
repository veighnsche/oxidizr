#[test]
#[ignore = "see BUGS.md:trybuild-compile-fail"]
fn compile_fail_on_atom_imports() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/*.rs");
}
