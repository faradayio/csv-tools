//! Integration tests for our CLI.

use cli_test_dir::*;

#[test]
fn help_flag() {
    let testdir = TestDir::new("scrubcsv", "flag_help");
    let output = testdir.cmd().arg("--help").expect_success();
    assert!(output.stdout_str().contains("scrubcsv"));
    assert!(output.stdout_str().contains("--help"));
}

#[test]
fn version_flag() {
    let testdir = TestDir::new("scrubcsv", "flag_version");
    let output = testdir.cmd().arg("--version").expect_success();
    assert!(output.stdout_str().contains("scrubcsv "));
}

#[test]
fn basic_file_scrubbing() {
    let testdir = TestDir::new("scrubcsv", "basic_scrubbing");
    testdir.create_file(
        "in.csv",
        "\
a,b,c
1,\"2\",3
\"Paris, France\",\"Broken \" quotes\",
",
    );
    let output = testdir.cmd().arg("in.csv").expect_success();
    // We reserve the right to change the exact output we generate for "Broken
    // \" quotes".  We could do a better job of guessing here.
    assert_eq!(
        output.stdout_str(),
        "\
a,b,c
1,2,3
\"Paris, France\",\"Broken  quotes\"\"\",
"
    );
    assert!(output.stderr_str().contains("3 rows (0 bad)"));
}

#[test]
fn stdin_and_delimiter_and_quiet() {
    let testdir = TestDir::new("scrubcsv", "stdin_and_delimiter_and_quiet");
    let output = testdir
        .cmd()
        .args(&["-d", "|"])
        .arg("-q")
        .output_with_stdin(
            "\
a|b|c
1|2|3
",
        )
        .expect_success();
    assert_eq!(
        output.stdout_str(),
        "\
a,b,c
1,2,3
"
    );
    assert!(!output.stderr_str().contains("rows"));
}

#[test]
fn quote_and_delimiter() {
    let testdir = TestDir::new("scrubcsv", "basic_scrubbing");
    testdir.create_file(
        "in.csv",
        "\
a\tb\tc
1\t\"2\t3
",
    );
    let output = testdir
        .cmd()
        .args(&["-d", r"\t"])
        .args(&["--quote", "none"])
        .arg("in.csv")
        .expect_success();
    assert_eq!(
        output.stdout_str(),
        "\
a,b,c
1,\"\"\"2\",3
"
    );
}

#[test]
fn bad_rows() {
    // Create a file with lots of good rows--enough to avoid triggering the
    // "too many bad rows" detection. This is an inefficient use of
    // `put_str`, but it doesn't matter for a test.
    let mut good_rows = "a,b,c\n".to_owned();
    for _ in 0..100 {
        good_rows.push_str("1,2,3\n");
    }
    let mut bad_rows = good_rows.clone();
    bad_rows.push_str("1,2\n");

    let testdir = TestDir::new("scrubcsv", "bad_rows");
    let output = testdir.cmd().output_with_stdin(&bad_rows).expect_success();
    assert_eq!(output.stdout_str(), &good_rows);
    assert!(output.stderr_str().contains("102 rows (1 bad)"));
}

#[test]
fn too_many_bad_rows() {
    let testdir = TestDir::new("scrubcsv", "too_many_bad_rows");
    let output = testdir
        .cmd()
        .output_with_stdin(
            "\
a,b,c
1,2
",
        )
        .expect("could not run scrubcsv");
    assert!(!output.status.success());
    assert_eq!(output.stdout_str(), "a,b,c\n");
    assert!(output
        .stderr_str()
        .contains("Too many rows (1 of 2) were bad"));
}

#[test]
fn null_normalization() {
    let testdir = TestDir::new("scrubcsv", "null_normalization");
    let output = testdir
        .cmd()
        .args(&["--null", "(?i)null|NIL"])
        .output_with_stdin("a,b,c,d,e\nnull,NIL,nil,,not null\n")
        .expect_success();
    assert_eq!(output.stdout_str(), "a,b,c,d,e\n,,,,not null\n")
}

#[test]
fn null_normalization_of_null_bytes() {
    let testdir = TestDir::new("scrubcsv", "null_normalization_of_null_bytes");
    let output = testdir
        .cmd()
        .args(&["--null", "\\x00"])
        .output_with_stdin("a,b\n\0,\n")
        .expect_success();
    assert_eq!(output.stdout_str(), "a,b\n,\n")
}

#[test]
fn replace_newlines() {
    let testdir = TestDir::new("scrubcsv", "replace_newlines");
    let output = testdir
        .cmd()
        .arg("--replace-newlines")
        .output_with_stdin("a,b\n\"line\r\nbreak\r1\",\"line\nbreak\n2\"\n")
        .expect_success();
    assert_eq!(output.stdout_str(), "a,b\nline break 1,line break 2\n");
}

#[test]
fn trim_whitespace() {
    let testdir = TestDir::new("scrubcsv", "trim_whitespace");
    let output = testdir
        .cmd()
        .arg("--trim-whitespace")
        .output_with_stdin("a,b,c,d\n 1 , 2, ,\n")
        .expect_success();
    assert_eq!(output.stdout_str(), "a,b,c,d\n1,2,,\n");
}

#[test]
fn clean_column_names_unique() {
    let testdir = TestDir::new("scrubcsv", "clean_column_names_unique");
    let output = testdir
        .cmd()
        .arg("--clean-column-names")
        .output_with_stdin(",,a,a\n")
        .expect_success();
    assert_eq!(output.stdout_str(), "_,__2,a,a_2\n");
}

#[test]
fn clean_column_names_stable() {
    let testdir = TestDir::new("scrubcsv", "clean_column_names_stable");
    let output = testdir
        .cmd()
        .arg("--clean-column-names=stable")
        .output_with_stdin("a,B,C d\n")
        .expect_success();
    assert_eq!(output.stdout_str(), "a,b,c_d\n");
}

#[test]
fn clean_column_names_stable_rejects_certain_names() {
    let testdir = TestDir::new(
        "scrubcsv",
        "clean_column_names_stable_rejects_certain_names",
    );

    let invalid_column_names = &[
        ("a,\n", "invalid column name"),
        ("1\n", "invalid column name"),
        ("A,a\n", "conflicting column names"),
        ("A b,a_b", "conflicting column names"),
    ];

    for &(names, err) in invalid_column_names {
        let output = testdir
            .cmd()
            .arg("--clean-column-names=stable")
            .output_with_stdin(names)
            .expect_failure();
        assert!(output.stderr_str().contains(err));
    }
}

#[test]
fn reserve_column_names() {
    let testdir = TestDir::new("scrubcsv", "clean_column_names_stable");
    let output = testdir
        .cmd()
        .arg("--clean-column-names=stable")
        .arg("--reserve-column-names=^reserved_")
        .output_with_stdin("a,Reserved Name\n")
        .expect_failure();
    assert!(output.stderr_str().contains("reserved column name"));
}

#[test]
fn drop_row_if_null() {
    let testdir = TestDir::new("scrubcsv", "replace_newlines");
    let output = testdir
        .cmd()
        .arg("--drop-row-if-null=c1")
        .arg("--drop-row-if-null=c2")
        .args(&["--null", "NULL"])
        .output_with_stdin(
            r#"c1,c2,c3
1,,
,2,
NULL,3,
a,b,c
"#,
        )
        .expect("error running scrubcsv");
    eprintln!("{}", output.stderr_str());
    //assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout_str(),
        r#"c1,c2,c3
a,b,c
"#
    );
}

#[test]
fn output_stats_json_format() {
    let testdir = TestDir::new("scrubcsv", "output_stats_json");
    testdir.create_file(
        "input.csv",
        "\
a,b,c
1,2,3
4,5,6
",
    );
    let output = testdir
        .cmd()
        .args(&["--output-stats", "stats.json", "--output-format", "json"])
        .arg("input.csv")
        .expect_success();

    // Check that CSV output is still correct
    assert_eq!(
        output.stdout_str(),
        "\
a,b,c
1,2,3
4,5,6
"
    );

    // Check that stats were written to file
    let stats_content = std::fs::read_to_string(testdir.path("stats.json"))
        .expect("Failed to read stats.json");
    assert!(stats_content.contains(r#""rows": 3"#));
    assert!(stats_content.contains(r#""bad_rows": 0"#));
    assert!(stats_content.contains(r#""elapsed_seconds""#));
    assert!(stats_content.contains(r#""bytes_per_second""#));

    // Verify it's valid JSON structure
    assert!(stats_content.starts_with("{"));
    assert!(stats_content.trim_end().ends_with("}"));
}

#[test]
fn output_stats_text_format() {
    let testdir = TestDir::new("scrubcsv", "output_stats_text");
    testdir.create_file(
        "input.csv",
        "\
name,age
Alice,25
Bob,30
",
    );
    let output = testdir
        .cmd()
        .args(&["--output-stats", "stats.txt", "--output-format", "text"])
        .arg("input.csv")
        .expect_success();

    // Check that CSV output is still correct
    assert_eq!(
        output.stdout_str(),
        "\
name,age
Alice,25
Bob,30
"
    );

    // Check that stats were written to file in text format
    let stats_content = std::fs::read_to_string(testdir.path("stats.txt"))
        .expect("Failed to read stats.txt");
    assert!(stats_content.contains("3 rows (0 bad)"));
    assert!(stats_content.contains("seconds"));
    assert!(stats_content.contains("/sec"));

    // Verify it matches the stderr format pattern
    assert!(!stats_content.starts_with("{"));  // Not JSON
}

#[test]
fn output_format_requires_output_stats() {
    let testdir = TestDir::new("scrubcsv", "output_format_dependency");
    testdir.create_file("input.csv", "a,b,c\n1,2,3\n");

    // This should fail because --output-format requires --output-stats
    let output = testdir
        .cmd()
        .args(&["--output-format", "json"])
        .arg("input.csv")
        .expect_failure();

    assert!(output.stderr_str().contains("--output-stats"));
}
