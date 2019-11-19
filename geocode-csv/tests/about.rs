//! Information about this CLI tool.

use cli_test_dir::*;

#[test]
fn help_flag() {
    let testdir = TestDir::new("geocode-csv", "help_flag");
    let output = testdir.cmd().arg("--help").output().expect_success();
    assert!(output.stdout_str().contains("geocode-csv"));
    assert!(output.stdout_str().contains("--help"));
}

#[test]
fn version_flag() {
    let testdir = TestDir::new("geocode-csv", "version_flag");
    let output = testdir.cmd().arg("--version").output().expect_success();
    assert!(output.stdout_str().contains("geocode-csv"));
    assert!(output.stdout_str().contains(env!("CARGO_PKG_VERSION")));
}
