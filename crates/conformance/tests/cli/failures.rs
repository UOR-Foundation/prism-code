//! Typed failures: one registered ID and one test each (`CL-09` to `CL-18`).
//!
//! Every diagnostic is the closed `TextCliFailure` name of the SDK, printed as
//! `<program>: <Failure>[: <detail>]` on standard error with the exit status of
//! its class: the model's usage code, its input or output code, or the code of
//! its `error` outcome.

use std::ffi::OsString;
use std::fs::File;
use std::path::Path;
use std::process::Stdio;

use crate::fixture;
use crate::program::{args, run, run_with, Output, Scratch};

fn usage(arguments: &[&str], failure: &str) {
    let fixture = fixture();
    let binary = &fixture.cli.cargo_name;
    let output = run(&fixture.program, &args(arguments), b"");
    assert_eq!(
        output.code,
        i32::from(fixture.cli.usage_code),
        "{arguments:?}: {}",
        output.stderr_text()
    );
    assert!(
        output.stdout.is_empty(),
        "{arguments:?}: {}",
        output.stdout_text()
    );
    assert_eq!(
        output.stderr_text(),
        format!("{binary}: {failure}\nTry '{binary} --help'.\n"),
        "{arguments:?}"
    );
}

fn input_output(output: &Output, failure: &str) {
    let fixture = fixture();
    assert_eq!(
        output.code,
        i32::from(fixture.cli.io_code),
        "{}",
        output.stderr_text()
    );
    assert_eq!(
        output.stderr_text(),
        format!("{}: {failure}\n", fixture.cli.cargo_name)
    );
}

fn check_path(path: &Path) -> Vec<OsString> {
    vec![
        OsString::from(&fixture().cli.command),
        path.as_os_str().to_owned(),
    ]
}

/// `CL-09`: an unknown flag.
#[test]
fn an_unknown_flag_is_a_usage_failure_cl_09() {
    usage(&["--bogus"], "UnknownFlag: --bogus");
}

/// `CL-10`: a second operand.
#[test]
fn an_extra_operand_is_a_usage_failure_cl_10() {
    let command = fixture().cli.command.as_str();
    usage(&[command, "a", "b"], "ExtraArgument: b");
}

/// `CL-11`: no command.
#[test]
fn a_missing_command_is_a_usage_failure_cl_11() {
    usage(&[], "MissingCommand");
}

/// `CL-12`: a command the model does not declare.
#[test]
fn an_unknown_command_is_a_usage_failure_cl_12() {
    usage(&["bogus"], "UnknownCommand: bogus");
}

/// `CL-13`: an example the model does not declare.
#[test]
fn an_unknown_example_is_a_usage_failure_cl_13() {
    assert!(
        fixture()
            .cli
            .examples
            .iter()
            .all(|example| example.name != "nope"),
        "nope is a modeled example, so it cannot be the unknown one"
    );
    usage(&["examples", "nope"], "UnknownExample: nope");
}

/// `CL-14`: a path that does not exist.
#[test]
fn a_missing_file_is_an_input_failure_cl_14() {
    let fixture = fixture();
    let scratch = Scratch::new();
    let path = scratch.dir().join("absent");
    let output = run(&fixture.program, &check_path(&path), b"");
    assert!(output.stdout.is_empty(), "{}", output.stdout_text());
    input_output(&output, &format!("InputMissing: {}", path.display()));
}

/// `CL-15`: a path that cannot be opened. A path below a regular file fails
/// with ENOTDIR for every user, where a file without permission does not fail
/// for the superuser.
#[test]
fn an_unopenable_path_is_an_input_failure_cl_15() {
    let fixture = fixture();
    let scratch = Scratch::new();
    let file = scratch.file("script", &fixture.cli.vectors[0].request);
    let path = file.join("below");
    let output = run(&fixture.program, &check_path(&path), b"");
    assert!(output.stdout.is_empty(), "{}", output.stdout_text());
    input_output(&output, &format!("InputOpen: {}", path.display()));
}

/// `CL-16`: a directory and a symbolic link are not regular files.
#[test]
fn a_non_regular_file_is_an_input_failure_cl_16() {
    let fixture = fixture();
    let scratch = Scratch::new();
    let target = scratch.file("target", &fixture.cli.vectors[0].request);
    let link = scratch.dir().join("link");
    std::os::unix::fs::symlink(&target, &link)
        .unwrap_or_else(|error| panic!("{}: {error}", link.display()));
    for path in [scratch.dir(), link.as_path()] {
        let output = run(&fixture.program, &check_path(path), b"");
        assert!(output.stdout.is_empty(), "{}", output.stdout_text());
        input_output(&output, &format!("InputType: {}", path.display()));
    }
}

/// `CL-17`: reading fails after the request was opened. A directory as standard
/// input opens and then fails to read with EISDIR.
#[test]
fn a_failed_read_is_an_input_failure_cl_17() {
    let fixture = fixture();
    let scratch = Scratch::new();
    let directory = File::open(scratch.dir())
        .unwrap_or_else(|error| panic!("{}: {error}", scratch.dir().display()));
    let output = run_with(
        &fixture.program,
        &args(&[fixture.cli.command.as_str()]),
        Stdio::from(directory),
        Stdio::piped(),
    );
    assert!(output.stdout.is_empty(), "{}", output.stdout_text());
    input_output(&output, "InputRead");
}

/// `CL-18`: standard output is closed, then full.
#[test]
fn an_unwritable_report_is_an_output_failure_cl_18() {
    let fixture = fixture();
    let request = &fixture.cli.vectors[0].request;
    let check = args(&[fixture.cli.command.as_str()]);
    let scratch = Scratch::new();

    let (reader, writer) = std::io::pipe().unwrap_or_else(|error| panic!("pipe: {error}"));
    drop(reader);
    let closed = run_with(
        &fixture.program,
        &check,
        scratch.stdin(request),
        Stdio::from(writer),
    );
    input_output(&closed, "OutputWrite");

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap_or_else(|error| panic!("/dev/full: {error}"));
    let full = run_with(
        &fixture.program,
        &check,
        scratch.stdin(request),
        Stdio::from(full),
    );
    input_output(&full, "OutputWrite");
}
