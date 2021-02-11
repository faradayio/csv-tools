extern crate cli_test_dir;

use cli_test_dir::*;

#[test]
fn cat_a_csv_and_csv_sz() {
    let testdir = TestDir::new("catcsv", "cat_a_csv_and_csv_sz");
    let output = testdir
        .cmd()
        .arg(testdir.src_path("fixtures/test.csv"))
        .arg(testdir.src_path("fixtures/test.csv.sz"))
        .output()
        .expect_success();
    assert_eq!(output.stdout_str(),
               "\
col1,col2
a,b
a,b
");
}

#[test]
fn cat_a_csv_and_csv_sz_in_a_dir() {
    let testdir = TestDir::new("catcsv", "cat_a_csv_and_csv_sz_in_a_dir");
    let output = testdir
        .cmd()
        .arg(testdir.src_path("fixtures"))
        .output()
        .expect_success();
    assert_eq!(output.stdout_str(),
               "\
col1,col2
a,b
a,b
");
}
