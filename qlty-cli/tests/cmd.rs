use crate::helpers::{setup_and_run_diff_test_cases, setup_and_run_test_cases};
use trycmd::TestCases;

#[test]
fn version_tests() {
    TestCases::new().case("qlty/tests/cmd/version/**/*.toml");
}

#[test]
fn help_tests() {
    TestCases::new().case("qlty/tests/cmd/help/**/*.toml");
}

#[test]
fn metrics_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/metrics/**/*.toml");
}

#[test]
fn duplication_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/duplication/**/*.toml");
}

#[test]
fn check_tests() {
    // only run .toml files in check directory
    // prevent running toml files from *.in
    setup_and_run_test_cases("qlty/tests/cmd/check/*.toml");
}

#[test]
#[ignore] // ignore tests that may require network connection
fn network_check_tests() {
    // only run .toml files in check/network directory
    // prevent running toml files from *.in
    setup_and_run_test_cases("qlty/tests/cmd/check/network/*.toml");
}

#[test]
#[ignore] // also may require network connection
fn fmt_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/fmt/**/*.toml");
}

#[test]
fn smells_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/smells/**/*.toml");
}

#[test]
fn coverage_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/coverage/**/*.toml");
}

#[test]
fn build_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/build/**/*.toml");
}

#[test]
fn init_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/init/*.toml");
}

#[test]
fn config_migrate_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/config/migrate/*.toml");
}

#[test]
#[ignore] // ignore tests that require network connection
fn init_network_tests() {
    setup_and_run_test_cases("qlty/tests/cmd/init/network/*.toml");
}

#[test]
fn git_based_check_tests() {
    setup_and_run_diff_test_cases("qlty/tests/cmd/check/diff_tests/*.toml");
}