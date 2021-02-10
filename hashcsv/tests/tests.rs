//! Integration tests for our CLI.

extern crate cli_test_dir;

use cli_test_dir::*;

#[test]
fn help_flag() {
    let testdir = TestDir::new("hashcsv", "flag_help");
    let output = testdir.cmd().arg("--help").expect_success();
    assert!(output.stdout_str().contains("hashcsv"));
    assert!(output.stdout_str().contains("--help"));
}

#[test]
fn version_flag() {
    let testdir = TestDir::new("hashcsv", "flag_version");
    let output = testdir.cmd().arg("--version").expect_success();
    assert!(output.stdout_str().contains("hashcsv "));
}

#[test]
fn assigns_hashes_based_on_row_contents() {
    let testdir = TestDir::new("hashcsv", "assigns_hashes_based_on_row_contents");
    let output = testdir
        .cmd()
        .output_with_stdin(
            "\
a,b,c
1,2,3
1,2,3
4,5,6
",
        )
        .expect_success();
    assert_eq!(
        output.stdout_str(),
        "\
a,b,c,id
1,2,3,ab37bf3a-c35c-51a9-802d-8eda9ee2f50a
1,2,3,ab37bf3a-c35c-51a9-802d-8eda9ee2f50a
4,5,6,481492ee-82c7-58b9-95ec-d92cbcd332c4
"
    );
}

#[test]
fn allows_setting_column_name() {
    let testdir = TestDir::new("hashcsv", "allows_setting_column_name");
    let output = testdir
        .cmd()
        .args(&["-c", "hash"])
        .output_with_stdin(
            "\
a,b,c
1,2,3
",
        )
        .expect_success();
    assert_eq!(
        output.stdout_str(),
        "\
a,b,c,hash
1,2,3,ab37bf3a-c35c-51a9-802d-8eda9ee2f50a
"
    );
}