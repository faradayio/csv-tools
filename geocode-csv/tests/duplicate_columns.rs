//! Specifying how to handle duplicate columns.

use cli_test_dir::*;

/// A CSV file to use for our tests.
const SIMPLE_CSV: &str = r#"address,gc_addressee,zip
20 W 34th St,,10118
"#;

/// A spec file to use for our tests.
const SIMPLE_SPEC: &str = r#"{
    "gc": {
        "house_number_and_street": "address",
        "postcode": "zip"
    }
}"#;

#[test]
#[ignore]
fn duplicate_columns_error() {
    let testdir = TestDir::new("geocode-csv", "duplicate_columns_error");

    testdir.create_file("spec.json", SIMPLE_SPEC);
    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .arg("--duplicate-columns=error")
        .output_with_stdin(SIMPLE_CSV)
        .expect("could not run geocode-csv");

    assert!(!output.status.success());
}

#[test]
#[ignore]
fn duplicate_columns_replace() {
    let testdir = TestDir::new("geocode-csv", "duplicate_columns_replace");

    testdir.create_file("spec.json", SIMPLE_SPEC);
    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .arg("--duplicate-columns=replace")
        .output_with_stdin(SIMPLE_CSV)
        .expect_success();

    assert!(output.stdout_str().contains("address,zip,gc_addressee,"));
    assert!(output.stdout_str().contains("Commercial"));
}

#[test]
#[ignore]
fn duplicate_columns_append() {
    let testdir = TestDir::new("geocode-csv", "duplicate_columns_append");

    testdir.create_file("spec.json", SIMPLE_SPEC);
    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .arg("--duplicate-columns=append")
        .output_with_stdin(SIMPLE_CSV)
        .expect_success();

    assert!(output
        .stdout_str()
        .contains("address,gc_addressee,zip,gc_addressee,"));
    assert!(output.stdout_str().contains("Commercial"));
}
