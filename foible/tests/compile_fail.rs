// The expected `.stderr` files are rustc's wording, so they can need
// regenerating (`TRYBUILD=overwrite cargo test`) after a toolchain upgrade.
#[test]
fn compile_fail() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
