use assert_cmd::Command;
use expectrl::{Any, Expect, spawn};
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
fn when_user_enter_a_numeric_ascii_code_value_then_cli_do_not_show_an_error() {
    let mut session = spawn("cargo run -- run -f ./resources/integration/test_input.txt")
        .expect("Error runing the run subcommand");

    session
        .expect(Any::boxed(vec![Box::new(
            "Write an ascii character or his code value",
        )]))
        .expect("Message waiting to the input not rendered");

    session
        .send_line("65")
        .expect("Failed to write the numeric ascii code");

    let value = session
        .expect(Any::boxed(vec![Box::new("'A'")]))
        .expect("Display cell value not redered");

    let message_end = session
        .expect(Any::boxed(vec![Box::new("Program executed succesfully")]))
        .expect("CLI not ended succesfully");

    assert_eq!(String::from_utf8_lossy(value.get(0).unwrap()), "'A'");
    assert_eq!(
        String::from_utf8_lossy(message_end.get(0).unwrap()),
        "Program executed succesfully"
    );
}

#[test]
fn when_user_enter_an_ascii_char_then_cli_do_not_show_an_error() {
    let mut session = spawn("cargo run -- run -f ./resources/integration/test_input.txt")
        .expect("Error runing the run subcommand");

    session
        .expect(Any::boxed(vec![Box::new(
            "Write an ascii character or his code value",
        )]))
        .expect("Message waiting to the input not rendered");

    session
        .send_line("B")
        .expect("Failed to write the numeric ascii code");

    let value = session
        .expect(Any::boxed(vec![Box::new("'B'")]))
        .expect("Display cell value not redered");

    let message_end = session
        .expect(Any::boxed(vec![Box::new("Program executed succesfully")]))
        .expect("CLI not ended succesfully");

    assert_eq!(String::from_utf8_lossy(value.get(0).unwrap()), "'B'");
    assert_eq!(
        String::from_utf8_lossy(message_end.get(0).unwrap()),
        "Program executed succesfully"
    );
}

#[test]
fn when_user_enter_an_invalid_ascii_char_code_then_cli_show_invalid_error() {
    let mut session = spawn("cargo run -- run -f ./resources/integration/test_input.txt")
        .expect("Error runing the run subcommand");

    session
        .expect(Any::boxed(vec![Box::new(
            "Write an ascii character or his code value",
        )]))
        .expect("Message waiting to the input not rendered");

    session
        .send_line("128")
        .expect("Failed to write the numeric ascii code");

    let error_message = session
        .expect(Any::boxed(vec![Box::new(
            "Please type a valid ascii character",
        )]))
        .expect("Error message not rendered");

    assert_eq!(
        String::from_utf8_lossy(error_message.get(0).unwrap()),
        "Please type a valid ascii character"
    );
}

#[test]
fn when_user_enter_an_invalid_ascii_char_then_cli_show_invalid_error() {
    let mut session = spawn("cargo run -- run -f ./resources/integration/test_input.txt")
        .expect("Error runing the run subcommand");

    session
        .expect(Any::boxed(vec![Box::new(
            "Write an ascii character or his code value",
        )]))
        .expect("Message waiting to the input not rendered");

    session
        .send_line("Ñ")
        .expect("Failed to write the numeric ascii code");

    let error_message = session
        .expect(Any::boxed(vec![Box::new(
            "Please type a valid ascii character",
        )]))
        .expect("Error message not rendered");

    assert_eq!(
        String::from_utf8_lossy(error_message.get(0).unwrap()),
        "Please type a valid ascii character"
    );
}

#[test]
fn when_running_hello_world_source_code_then_render_hello_world_and_complete_successfully() {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args(["run", "-f", file_test_case!("test_hello_world.txt")])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("'H''e''l''l''o'' ''W''o''r''l''d''!'")
                .and(predicate::str::contains("Program executed succesfully")),
        );
}

#[test]
fn when_running_a_source_code_with_infinite_loop_then_render_error_of_unable_to_complete_the_program()
 {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args(["run", "-f", file_test_case!("test_infinite_loop.txt")])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Not enought reads to complete the program. Check if the program have infinite loops or increased the amount of reads")

        );
}

#[test]
fn when_the_amount_of_reads_provided_is_lower_than_1_then_render_an_error_of_invalid_argument_value()
 {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args([
            "run",
            "-l",
            "0",
            "-f",
            file_test_case!("test_hello_world.txt"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value '0' for '-l <limit-read-instructions>': Minimum value accepted is 1",
        ));
}

#[test]
fn when_the_amount_of_reads_provided_is_greater_than_the_maximum_of_reads_then_render_an_error_of_invalid_argument_value()
 {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args([
            "run",
            "-l",
            "100001",
            "-f",
            file_test_case!("test_hello_world.txt"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value '100001' for '-l <limit-read-instructions>': Maximum value accepted is 100_000",
        ));
}

#[test]
fn when_running_a_source_code_with_lack_of_open_brackets_then_render_error_of_unable_to_complete_the_program()
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
fn when_running_a_source_code_with_lack_of_closing_brackets_then_render_error_of_unable_to_complete_the_program()
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

#[test]
fn give_a_memory_size_smaller_when_the_program_moves_to_position_out_of_bounds_then_render_a_runtime_error()
 {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args([
            "run",
            "-m",
            "2",
            "-f",
            file_test_case!("test_hello_world.txt"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The program is trying to access to position out of range in the memory",
        ));
}

#[test]
fn when_the_memory_size_is_lower_than_1_then_cli_return_an_error_as_the_argument_value_is_invalid()
{
    Command::cargo_bin("braincrab")
        .unwrap()
        .args([
            "run",
            "-m",
            "0",
            "-f",
            file_test_case!("test_hello_world.txt"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value '0' for '-m <memory-size>': Minimum value accepted is 1",
        ));
}

#[test]
fn when_the_memory_size_is_greather_than_the_maximum_memory_size_then_cli_return_an_error_as_the_argument_value_is_invalid()
 {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args([
            "run",
            "-m",
            "100001",
            "-f",
            file_test_case!("test_hello_world.txt"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value '100001' for '-m <memory-size>': Maximum value accepted is 30_000",
        ));
}
