//! Specifying columns to geocode.

use cli_test_dir::*;

/// A CSV file to geocode. Contains the empire state building.
const SIMPLE_CSV: &str = "address_1,address_2,city,state,zip_code
20 W 34th St,,New York,NY,10118
";

#[test]
#[ignore]
fn all_fields() {
    let testdir = TestDir::new("geocode-csv", "all_fields");

    testdir.create_file(
        "spec.json",
        r#"{
    "gc": {
        "house_number_and_street": [
            "address_1",
            "address_2"
        ],
        "city": "city",
        "state": "state",
        "postcode": "zip_code"
    }
}"#,
    );

    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .output_with_stdin(SIMPLE_CSV)
        .expect_success();
    assert!(output.stdout_str().contains("gc_addressee"));
    assert!(output.stdout_str().contains("Commercial"));
}

#[test]
#[ignore]
fn single_address_field() {
    let testdir = TestDir::new("geocode-csv", "single_address_field");

    testdir.create_file(
        "spec.json",
        r#"{
    "gc": {
        "house_number_and_street": "address_1",
        "city": "city",
        "state": "state",
        "postcode": "zip_code"
    }
}"#,
    );

    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .output_with_stdin(SIMPLE_CSV)
        .expect_success();
    assert!(output.stdout_str().contains("gc_addressee"));
    assert!(output.stdout_str().contains("Commercial"));
}

#[test]
#[ignore]
fn no_city_or_state() {
    let testdir = TestDir::new("geocode-csv", "no_city_or_state");

    testdir.create_file(
        "spec.json",
        r#"{
    "gc": {
        "house_number_and_street": [
            "address_1",
            "address_2"
        ],
        "postcode": "zip_code"
    }
}"#,
    );

    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .output_with_stdin(SIMPLE_CSV)
        .expect_success();
    assert!(output.stdout_str().contains("gc_addressee"));
    assert!(output.stdout_str().contains("Commercial"));
}

#[test]
#[ignore]
fn freeform() {
    let testdir = TestDir::new("geocode-csv", "freeform");

    testdir.create_file(
        "spec.json",
        r#"{
    "gc": {
        "house_number_and_street": [
            "address_1",
            "address_2",
            "city",
            "state",
            "zip_code"
        ]
    }
}"#,
    );

    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .output_with_stdin(SIMPLE_CSV)
        .expect_success();
    assert!(output.stdout_str().contains("gc_addressee"));
    assert!(output.stdout_str().contains("Commercial"));
}

#[test]
#[ignore]
fn multiple_addresses() {
    let testdir = TestDir::new("geocode-csv", "multiple_addresses");

    testdir.create_file(
        "spec.json",
        r#"{
    "shipping": {
        "house_number_and_street": [
            "address_1",
            "address_2"
        ],
        "postcode": "zip_code"
    },
    "billing": {
        "house_number_and_street": [
            "address_1",
            "address_2"
        ],
        "postcode": "zip_code"
    }
}"#,
    );

    let output = testdir
        .cmd()
        .arg("--spec=spec.json")
        .output_with_stdin(SIMPLE_CSV)
        .expect_success();
    assert!(output.stdout_str().contains("shipping_addressee"));
    assert!(output.stdout_str().contains("billing_addressee"));
}
