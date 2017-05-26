extern crate cli_test_dir;

use cli_test_dir::*;

#[test]
fn help_flag() {
    let testdir = TestDir::new("geochunk", "help_flag");
    let output = testdir.cmd().arg("--help").expect_success();
    assert!(output.stdout_str().contains("geochunk csv"));
}

#[test]
fn version_flag() {
    let testdir = TestDir::new("geochunk", "version_flag");
    let output = testdir.cmd().arg("--version").expect_success();
    assert!(output.stdout_str().contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn export_zip2010_outputs_csv() {
    let testdir = TestDir::new("geochunk", "version_flag");
    let output = testdir.cmd().args(&["export", "zip2010", "250000"]).expect_success();
    assert!(output.stdout_str().contains("zip,geochunk_zip2010_250000"));
    assert!(output.stdout_str().contains("01830,018_1"));
}
