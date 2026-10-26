//! What the capability suites share: building a request, running the program
//! on it and holding the bytes it prints to the expected report.
//!
//! The header and the report prefixes come from the acceptance vectors and the
//! exit-code map of the model, so this repository names none of them (`CL-04`).
//! A request that an acceptance vector already carries is tied to that vector,
//! so the expectation is the model's response and not a second opinion of it.

use crate::fixture;
use crate::model::{Cli, Vector};
use crate::program::{args, run, Output};

/// The script header the model's acceptance vectors begin with.
pub fn header(cli: &Cli) -> String {
    let request = cli
        .vectors
        .first()
        .expect("the model declares no acceptance vector")
        .request
        .as_slice();
    let first = request
        .split(|byte| *byte == b'\n')
        .next()
        .expect("split yields at least one piece");
    String::from_utf8(first.to_vec()).expect("the first vector begins with a text header")
}

/// A request: the header, then `lines`, joined by newlines with no trailing one.
pub fn script(cli: &Cli, lines: &[&str]) -> Vec<u8> {
    let mut text = header(cli);
    for line in lines {
        text.push('\n');
        text.push_str(line);
    }
    text.into_bytes()
}

fn report(cli: &Cli, outcome: &str, body: &[String]) -> Vec<u8> {
    let mut text = cli.outcome(outcome).prefix.clone();
    for line in body {
        text.push('\n');
        text.push_str(line);
    }
    text.into_bytes()
}

/// The report of an accepted project whose canonical form is `canonical`.
pub fn accepted(cli: &Cli, canonical: &[&str]) -> Vec<u8> {
    let mut body = vec![header(cli)];
    body.extend(canonical.iter().map(|line| (*line).to_owned()));
    report(cli, "accepted", &body)
}

/// The report of a rejected project: its `findings`, then its canonical form.
pub fn rejected(cli: &Cli, findings: &[&str], canonical: &[&str]) -> Vec<u8> {
    let mut body: Vec<String> = findings.iter().map(|line| (*line).to_owned()).collect();
    body.push(header(cli));
    body.extend(canonical.iter().map(|line| (*line).to_owned()));
    report(cli, "rejected", &body)
}

/// The report of a request refused at `line` with `code`.
pub fn failed(cli: &Cli, line: usize, code: &str) -> Vec<u8> {
    report(cli, "error", &[format!("line {line} {code}")])
}

/// The report of a request refused as a whole, as `request empty`.
pub fn refused(cli: &Cli, reason: &str) -> Vec<u8> {
    report(cli, "error", &[format!("request {reason}")])
}

/// Run `check` on `request` from standard input.
pub fn check(request: &[u8]) -> Output {
    let fixture = fixture();
    run(
        &fixture.program,
        &args(&[fixture.cli.command.as_str()]),
        request,
    )
}

fn show(bytes: &[u8]) -> String {
    format!(
        "{:?} ({} bytes)",
        String::from_utf8_lossy(bytes),
        bytes.len()
    )
}

/// What disagrees between the program's answer to `request` and `expected`:
/// the printed bytes, the exit status of the outcome the report begins with,
/// and standard error. Empty when the program answered exactly.
pub fn disagreements(request: &[u8], expected: &[u8]) -> Vec<String> {
    let cli = &fixture().cli;
    let output = check(request);
    let mut found = Vec::new();
    if output.stdout != expected {
        found.push(format!(
            "request {}: printed {}, expected {}",
            show(request),
            show(&output.stdout),
            show(expected)
        ));
    }
    match cli.classify(expected) {
        Some(outcome) if output.code == i32::from(outcome.exit_code) => {}
        Some(outcome) => found.push(format!(
            "request {}: exit status {}, expected {} for {}",
            show(request),
            output.code,
            outcome.exit_code,
            outcome.name
        )),
        None => found.push(format!(
            "the expected report {} has no declared outcome",
            show(expected)
        )),
    }
    if !output.stderr.is_empty() {
        found.push(format!(
            "request {}: standard error {}",
            show(request),
            show(&output.stderr)
        ));
    }
    found
}

/// Assert that the program answers `request` with exactly `expected`.
pub fn expect(request: &[u8], expected: &[u8]) {
    let found = disagreements(request, expected);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// The acceptance vector whose request is exactly `request`.
pub fn vector_of<'a>(cli: &'a Cli, request: &[u8]) -> Option<&'a Vector> {
    cli.vectors.iter().find(|vector| vector.request == request)
}

/// Assert that the model carries an acceptance vector for `request` and that
/// its response is `expected`, so the expectation below is the model's.
pub fn assert_modeled(request: &[u8], expected: &[u8]) {
    let cli = &fixture().cli;
    let vector = vector_of(cli, request)
        .unwrap_or_else(|| panic!("no acceptance vector has the request {}", show(request)));
    assert_eq!(
        show(&vector.response),
        show(expected),
        "the modeled response differs for the request {}",
        show(request)
    );
}

/// Expect the answer to a script and require the model to carry it as a vector.
pub fn expect_vector(lines: &[&str], expected: &[u8]) {
    let request = script(&fixture().cli, lines);
    assert_modeled(&request, expected);
    expect(&request, expected);
}

/// Expect the answer to a script that no acceptance vector carries.
pub fn expect_script(lines: &[&str], expected: &[u8]) {
    expect(&script(&fixture().cli, lines), expected);
}

/// Expect the refusal of a script at `line` with `code`, as a vector.
pub fn vector_refusal(lines: &[&str], line: usize, code: &str) {
    expect_vector(lines, &failed(&fixture().cli, line, code));
}

/// Expect the refusal of a script at `line` with `code`.
pub fn script_refusal(lines: &[&str], line: usize, code: &str) {
    expect_script(lines, &failed(&fixture().cli, line, code));
}
