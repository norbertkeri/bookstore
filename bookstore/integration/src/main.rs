use testharness::run_tests;

pub mod bookstore_test;
pub mod testharness;

fn main() -> color_eyre::Result<()> {
    run_tests("integration", "debug")
}
