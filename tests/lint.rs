use assert_cmd::Command;
use predicates::prelude::*;

macro_rules! file_test_case {
    ($fname:expr) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/integration/",
            $fname
        )
    };
}

#[test]
fn when_linting_code_well_written_then_render_succesfully_linted() {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args(["lint", "-f", file_test_case!("test_hello_world.txt")])
        .assert()
        .success()
        .stdout(predicate::str::contains("All good!"));
}

#[test]
fn when_linting_a_source_code_with_lack_of_open_brackets_then_render_error_of_unable_to_complete_the_program()
 {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args(["run", "-f", file_test_case!("test_lack_open_loop.txt")])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The source code have more closing loop brackets than open loop brackets.",
        ));
}

#[test]
fn when_linting_a_source_code_with_lack_of_closing_brackets_then_render_error_of_unable_to_complete_the_program()
 {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args(["run", "-f", file_test_case!("test_lack_close_loop.txt")])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The source code have more open loop brackets than closing loop brackets.",
        ));
}
