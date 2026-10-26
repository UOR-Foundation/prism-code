//! `check`: the request, the report and the exit status (`CL-01` to `CL-03`, `CL-07`).

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;

use crate::fixture;
use crate::program::{args, run, Scratch};

fn check_args(command: &str, operand: Option<&Path>) -> Vec<OsString> {
    let mut arguments = vec![OsString::from(command)];
    arguments.extend(operand.map(|path| path.as_os_str().to_owned()));
    arguments
}

/// `CL-01`: a file, `-` and standard input give the same bytes.
#[test]
fn check_reads_a_file_dash_and_stdin_alike_cl_01() {
    let fixture = fixture();
    let cli = &fixture.cli;
    let scratch = Scratch::new();
    let mut checked = 0;
    for (index, vector) in cli.vectors.iter().enumerate() {
        if vector.request.len() > cli.request_maximum || cli.is_framed(&vector.request) {
            continue;
        }
        let path = scratch.file("request", &vector.request);
        let runs = [
            ("a file", check_args(&cli.command, Some(&path)), Vec::new()),
            (
                "-",
                check_args(&cli.command, Some(Path::new("-"))),
                vector.request.clone(),
            ),
            (
                "standard input",
                check_args(&cli.command, None),
                vector.request.clone(),
            ),
        ];
        for (how, arguments, stdin) in runs {
            let output = run(&fixture.program, &arguments, &stdin);
            assert_eq!(
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&vector.response),
                "vector {index} read from {how}: {}",
                output.stderr_text()
            );
            assert!(
                output.stderr.is_empty(),
                "vector {index} read from {how} wrote to standard error: {}",
                output.stderr_text()
            );
        }
        checked += 1;
    }
    assert!(checked > 0, "the model declares no acceptance vector");
}

/// `CL-02`: the exit status is the modeled code of the report's outcome.
#[test]
fn exit_status_is_the_modeled_outcome_code_cl_02() {
    let fixture = fixture();
    let cli = &fixture.cli;
    // The map #17 requires. The program takes it from the model, so this is the
    // check that the model declares what the issue asks for.
    let declared: Vec<(&str, u8)> = cli
        .outcomes
        .iter()
        .map(|outcome| (outcome.name.as_str(), outcome.exit_code))
        .collect();
    assert_eq!(declared, [("accepted", 0), ("rejected", 1), ("error", 2)]);
    assert_eq!((cli.usage_code, cli.io_code), (64, 74));

    let mut seen = BTreeSet::new();
    for (index, vector) in cli.vectors.iter().enumerate() {
        if vector.request.len() > cli.request_maximum || cli.is_framed(&vector.request) {
            continue;
        }
        let outcome = cli
            .classify(&vector.response)
            .unwrap_or_else(|| panic!("vector {index} has a response with no declared outcome"));
        let output = run(
            &fixture.program,
            &check_args(&cli.command, None),
            &vector.request,
        );
        assert_eq!(
            output.code,
            i32::from(outcome.exit_code),
            "vector {index} is {}: {}",
            outcome.name,
            output.stderr_text()
        );
        seen.insert(outcome.name.clone());
    }
    let wanted: BTreeSet<String> = cli.outcomes.iter().map(|o| o.name.clone()).collect();
    assert_eq!(seen, wanted, "every outcome needs a vector that reaches it");
}

/// `CL-03`: a request over the maximum never reaches dispatch.
#[test]
fn an_oversized_request_is_refused_before_dispatch_cl_03() {
    let fixture = fixture();
    let cli = &fixture.cli;
    let mut requests: Vec<Vec<u8>> = cli
        .vectors
        .iter()
        .filter(|vector| vector.request.len() > cli.request_maximum)
        .map(|vector| vector.request.clone())
        .collect();
    assert!(
        !requests.is_empty(),
        "the model declares no vector over the request maximum"
    );
    requests.push(vec![b'a'; cli.request_maximum + 1]);
    let fault = i32::from(cli.limit_exit_code());
    let binary = &cli.cargo_name;
    let scratch = Scratch::new();
    for request in requests {
        let path = scratch.file("oversized", &request);
        let output = run(&fixture.program, &check_args(&cli.command, None), &request);
        assert_eq!(
            output.code,
            fault,
            "standard input: {}",
            output.stderr_text()
        );
        assert!(output.stdout.is_empty(), "{}", output.stdout_text());
        assert_eq!(output.stderr_text(), format!("{binary}: InputLimit\n"));

        let output = run(
            &fixture.program,
            &check_args(&cli.command, Some(&path)),
            b"",
        );
        assert_eq!(output.code, fault, "a file: {}", output.stderr_text());
        assert!(output.stdout.is_empty(), "{}", output.stdout_text());
        assert_eq!(
            output.stderr_text(),
            format!("{binary}: InputLimit: {}\n", path.display())
        );
    }
}

/// `CL-07`: examples are listed, printed unchanged and checkable.
#[test]
fn examples_are_listed_printed_and_checkable_cl_07() {
    let fixture = fixture();
    let (cli, guidance) = (&fixture.cli, &fixture.guidance);
    let listing = run(&fixture.program, &args(&["examples"]), b"");
    assert_eq!(listing.code, 0, "{}", listing.stderr_text());
    assert!(listing.stderr.is_empty(), "{}", listing.stderr_text());
    let names: String = cli
        .examples
        .iter()
        .map(|example| format!("{}\n", example.name))
        .collect();
    assert_eq!(listing.stdout_text(), names);

    let workbench: Vec<&str> = guidance.examples.iter().map(|e| e.name.as_str()).collect();
    let modeled: Vec<&str> = cli.examples.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        workbench, modeled,
        "the page and the command line offer the same examples"
    );

    for spec in &guidance.examples {
        let example = cli
            .examples
            .iter()
            .find(|example| example.name == spec.name)
            .unwrap_or_else(|| panic!("no command line example named {}", spec.name));
        let printed = run(&fixture.program, &args(&["examples", &spec.name]), b"");
        assert_eq!(printed.code, 0, "{}: {}", spec.name, printed.stderr_text());
        assert!(printed.stderr.is_empty(), "{}", printed.stderr_text());
        assert_eq!(printed.stdout_text(), example.text, "{}", spec.name);

        let vector = cli
            .vectors
            .get(spec.vector)
            .unwrap_or_else(|| panic!("{} names vector {}", spec.name, spec.vector));
        let checked = run(
            &fixture.program,
            &check_args(&cli.command, None),
            &printed.stdout,
        );
        assert_eq!(
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&vector.response),
            "{} piped to check: {}",
            spec.name,
            checked.stderr_text()
        );
        assert_eq!(
            checked.code,
            i32::from(cli.outcome(&spec.outcome).exit_code),
            "{} piped to check",
            spec.name
        );
    }
}
