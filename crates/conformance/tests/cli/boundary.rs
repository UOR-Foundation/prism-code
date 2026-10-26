//! The request and response boundaries of `dispatch` (`BD-01` to `BD-07`).
//!
//! A request is at most `requestMaximum` bytes and is split into at most
//! `lineMaximum` lines; a response is at most `responseMaximum` bytes. Two of
//! the six refusals, `request too-many-lines` and `response too-large`, cannot
//! be produced from any request within the request maximum, so their tests hold
//! the arithmetic over the modeled limits and the densest request instead of a
//! vector. The model carries no theorem for either bound: the proof language
//! has no lemma about the splitter or the encoder (`features/suites/README.md`).

use crate::evidence;
use crate::fixture;
use crate::model::Cli;
use crate::program::{args, run};
use crate::report::{assert_modeled, check, expect, failed, header, refused, script, vector_of};
use crate::root;

fn vector_with_length(cli: &Cli, length: usize) -> &crate::model::Vector {
    let mut found = cli
        .vectors
        .iter()
        .filter(|vector| vector.request.len() == length && !cli.is_framed(&vector.request));
    let vector = found
        .next()
        .unwrap_or_else(|| panic!("no acceptance vector has a request of {length} bytes"));
    assert!(found.next().is_none(), "two vectors have {length} bytes");
    vector
}

/// `BD-01`: an empty request.
#[test]
fn an_empty_request_is_refused_bd_01() {
    let cli = &fixture().cli;
    let vector = vector_with_length(cli, 0);
    assert_eq!(vector.response, refused(cli, "empty"));
    expect(b"", &refused(cli, "empty"));
}

/// `BD-02`: a request of exactly 4097 bytes is refused before it is read.
#[test]
fn a_request_of_4097_bytes_is_too_large_bd_02() {
    let fixture = fixture();
    let cli = &fixture.cli;
    assert_eq!(cli.request_maximum, 4096);
    let vector = vector_with_length(cli, 4097);
    assert_eq!(vector.response, refused(cli, "too-large"));

    // The command line refuses it before dispatch, and prismpm verify holds the
    // Core-Wasm entry and the browser view to the modeled response for it.
    let oversized = cli
        .vectors
        .iter()
        .filter(|vector| vector.request.len() > cli.request_maximum)
        .count();
    evidence::attestation(root(), &fixture.program).require(cli.vectors.len(), oversized);
    let output = check(&vector.request);
    assert!(output.stdout.is_empty(), "{}", output.stdout_text());
    assert_eq!(
        output.stderr_text(),
        format!("{}: InputLimit\n", cli.cargo_name)
    );
    assert_eq!(output.code, i32::from(cli.limit_exit_code()));

    // The size decides, not the content: a script that is valid up to its last
    // byte is refused as well, and so is one byte over the accepted request.
    let accepted = vector_with_length(cli, 4096);
    let mut longer = accepted.request.clone();
    longer.push(b'\n');
    assert_eq!(longer.len(), 4097);
    let output = check(&longer);
    assert!(output.stdout.is_empty(), "{}", output.stdout_text());
    assert_eq!(
        output.stderr_text(),
        format!("{}: InputLimit\n", cli.cargo_name)
    );
    assert_eq!(output.code, i32::from(cli.limit_exit_code()));
}

/// `BD-03`: bytes that are not UTF-8.
#[test]
fn a_request_that_is_not_utf8_is_refused_bd_03() {
    let cli = &fixture().cli;
    let vector = vector_with_length(cli, 1);
    assert_eq!(vector.response, refused(cli, "invalid-utf8"));
    expect(&vector.request, &refused(cli, "invalid-utf8"));
    let mut late = script(cli, &["node a input 0"]);
    late.push(b'\n');
    late.push(0xff);
    for request in [&[0xc3, 0x28][..], &[0xff, 0xfe][..], &late[..]] {
        expect(request, &refused(cli, "invalid-utf8"));
    }
}

/// `BD-04`: no request within the request maximum has more lines than the line
/// maximum, so `request too-many-lines` has no vector.
#[test]
fn a_request_cannot_exceed_the_line_maximum_within_the_byte_maximum_bd_04() {
    let cli = &fixture().cli;
    // N bytes hold at most N newlines, so at most N + 1 lines; the model's line
    // maximum is that bound for the request maximum.
    assert_eq!(cli.line_maximum, cli.request_maximum + 1);
    // The request with the most lines is all newlines: 4097 lines, which the
    // splitter still accepts, so the first line is what is refused.
    let densest = vec![b'\n'; cli.request_maximum];
    assert_eq!(
        densest.iter().filter(|byte| **byte == b'\n').count() + 1,
        cli.line_maximum
    );
    expect(&densest, &failed(cli, 1, "missing-header"));
    // The longest request that is a script, vector 12, has fewer lines still.
    let vector = vector_with_length(cli, cli.request_maximum);
    let lines = vector.request.iter().filter(|byte| **byte == b'\n').count() + 1;
    assert!(lines <= cli.line_maximum, "vector 12 has {lines} lines");
    expect(&vector.request, &vector.response);
    // One byte more is over the request maximum and never gets as far as
    // counting lines: it is refused as too large.
    let output = check(&vec![b'\n'; cli.request_maximum + 1]);
    assert!(output.stdout.is_empty(), "{}", output.stdout_text());
    assert_eq!(
        output.stderr_text(),
        format!("{}: InputLimit\n", cli.cargo_name)
    );
}

/// `BD-05`: a first line that is not the header.
#[test]
fn a_request_without_the_header_is_refused_bd_05() {
    let cli = &fixture().cli;
    let vector = vector_of(cli, b"node camera input 0")
        .unwrap_or_else(|| panic!("no acceptance vector lacks the header"));
    assert_eq!(vector.response, failed(cli, 1, "missing-header"));
    expect(&vector.request, &failed(cli, 1, "missing-header"));
    let header = header(cli);
    let mut other_version = header.clone();
    other_version.pop();
    other_version.push('2');
    for request in [
        other_version.clone(),
        format!("{header} "),
        header.to_uppercase(),
        format!("\n{header}"),
        format!("{header}x\nnode a input 0"),
    ] {
        expect(request.as_bytes(), &failed(cli, 1, "missing-header"));
    }
}

/// The longest request within the maximum that makes the response longest: one
/// input marked as PII feeding output nodes of increasing name length, each over
/// a memory limit of zero, so each output adds a node line, an edge line, a
/// memory finding and a plaintext-pii finding.
fn densest(cli: &Cli) -> (Vec<u8>, usize) {
    let mut text = String::from_utf8(script(
        cli,
        &[
            "policy memory 0",
            "policy encrypt-pii",
            "node a input 0",
            "pii a",
        ],
    ))
    .expect("a script is text");
    let mut outputs = 0;
    let mut length = 1;
    'grow: loop {
        for index in 0..26usize.pow(u32::try_from(length).expect("small")) {
            let mut name = String::new();
            let mut rest = index;
            for _ in 0..length {
                name.insert(
                    0,
                    char::from(b'a' + u8::try_from(rest % 26).expect("below 26")),
                );
                rest /= 26;
            }
            if name == "a" {
                continue;
            }
            let addition = format!("\nnode {name} output 1\nedge a {name}");
            if text.len() + addition.len() > cli.request_maximum {
                break 'grow;
            }
            text.push_str(&addition);
            outputs += 1;
        }
        length += 1;
    }
    (text.into_bytes(), outputs)
}

/// `BD-06`: a response over the response maximum would be replaced by
/// `response too-large`, and no request within the request maximum gets there.
#[test]
fn a_response_cannot_exceed_the_response_maximum_within_the_request_maximum_bd_06() {
    let cli = &fixture().cli;
    assert_eq!(cli.response_maximum, 16384);
    let (request, outputs) = densest(cli);
    assert!(request.len() <= cli.request_maximum);
    assert!(
        request.len() + 64 > cli.request_maximum,
        "the request is within 64 bytes of the maximum, so no output more fits"
    );
    let output = run(&fixture().program, &args(&[cli.command.as_str()]), &request);
    let report = output.stdout_text();
    let memory = report
        .lines()
        .filter(|line| line.starts_with("finding memory "))
        .count();
    let pii = report
        .lines()
        .filter(|line| line.starts_with("finding plaintext-pii "))
        .count();
    // Every output carries both findings, so the response is the densest kind.
    assert_eq!(
        (memory, pii),
        (outputs, outputs),
        "{}",
        output.stderr_text()
    );
    assert!(
        output.stdout.len() > 2 * request.len(),
        "the response is {} bytes for a request of {}",
        output.stdout.len(),
        request.len()
    );
    assert!(
        output.stdout.len() < cli.response_maximum,
        "the densest request gives {} bytes, over the {} maximum",
        output.stdout.len(),
        cli.response_maximum
    );
    assert!(output.stderr.is_empty(), "{}", output.stderr_text());
    assert_eq!(output.code, i32::from(cli.outcome("rejected").exit_code));
}

/// `BD-07`: a request of exactly 4096 bytes is accepted.
#[test]
fn a_request_of_4096_bytes_is_accepted_bd_07() {
    let cli = &fixture().cli;
    let vector = vector_with_length(cli, 4096);
    assert_eq!(vector.request.len(), cli.request_maximum);
    assert_modeled(&vector.request, &vector.response);
    expect(&vector.request, &vector.response);
    assert!(
        vector
            .response
            .starts_with(cli.outcome("accepted").prefix.as_bytes()),
        "vector 12 is accepted"
    );
    // One byte fewer is accepted too, so the limit is the byte count and not
    // the shape of the request.
    expect(&vector.request[..4095], &vector.response);
}
