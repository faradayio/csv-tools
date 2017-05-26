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
    let testdir = TestDir::new("geochunk", "export_zip2010_outputs_csv");
    let output = testdir
        .cmd()
        .args(&["export", "zip2010", "250000"])
        .expect_success();
    assert!(output
                .stdout_str()
                .contains("zip,geochunk_zip2010_250000"));
    assert!(output.stdout_str().contains("01830,018_1"));
}

#[test]
fn csv_zip2010_adds_column_to_csv_file() {
    let testdir = TestDir::new("geochunk", "export_zip2010_outputs_csv");
    let input = "\
name,postcode
J. Doe,90210
";
    let output = testdir
        .cmd()
        .args(&["csv", "zip2010", "250000", "postcode"])
        .output_with_stdin(input)
        .expect_success();
    assert_eq!(output.stdout_str(),
               "\
name,postcode,geochunk_zip2010_250000
J. Doe,90210,902_0
");
}
