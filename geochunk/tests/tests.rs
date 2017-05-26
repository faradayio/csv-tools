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
