use std::{
    fs::{File, remove_file},
    os::unix::fs::PermissionsExt,
};

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn when_user_pass_an_invalid_subcomamnd_then_render_the_subcommand_does_not_exist() {
    Command::cargo_bin("braincrab")
        .unwrap()
        .args(["none"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand 'none'"));
}

#[test]
fn given_user_with_lack_of_permission_when_user_try_to_read_a_file_without_permission_then_return_not_enough_permission_error()
 {
    let path_file = concat!(env!("CARGO_TARGET_TMPDIR"), "no_permission.txt");
    let file = File::create(path_file)
        .expect(format!("Unable to create file in CARGO_TARGET_TMPDIR ${path_file}").as_str());
    let mut permission = file.metadata().unwrap().permissions();

    permission.set_mode(0o000);

    file.set_permissions(permission)
        .expect("Unable to change the permission for the tmp file");

    Command::cargo_bin("braincrab")
        .unwrap()
        .args(["run", "-f", path_file])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Unable to read the file due lack of permission",
        ));

    remove_file(path_file).expect("File to be deleted")
}
