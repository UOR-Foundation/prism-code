//! Web and command-line parity (`PA-01`).
//!
//! The page and the command line call the same entry. `prismpm verify` holds
//! the Core-Wasm entry and the browser view to the modeled response for every
//! acceptance vector, so a command line whose bytes equal the modeled response
//! produces what the page does. The page is not run again here.
//!
//! Two vector classes have no response to compare, and are compared by what the
//! command line does instead, as the SDK specifies for text/3:
//!
//! - a request over the request maximum is refused before dispatch with
//!   `InputLimit`, while the page renders the modeled error report;
//! - a workbench frame is answered by the page in its own result protocol, and
//!   the command line refuses the report with `ReportUnclassified`.

use std::ffi::OsString;
use std::path::Path;

use crate::evidence;
use crate::model::Cli;
use crate::program::{run, Output, Scratch};
use crate::{fixture, root};

/// How each vector was compared.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    /// Response bytes and exit status compared with the modeled response.
    pub bytes: usize,
    /// Refused before dispatch because the request is over the maximum.
    pub oversized: usize,
    /// Refused because the page answers in the workbench protocol.
    pub framed: usize,
}

impl Coverage {
    /// Vectors compared by any means.
    pub fn total(&self) -> usize {
        self.bytes + self.oversized + self.framed
    }
}

fn show(bytes: &[u8]) -> String {
    format!(
        "{:?} ({} bytes)",
        String::from_utf8_lossy(bytes),
        bytes.len()
    )
}

/// Check one refusal: empty standard output, the diagnostic and the exit status.
fn refusal(output: &Output, binary: &str, name: &str, code: u8) -> Vec<String> {
    let mut found = Vec::new();
    if !output.stdout.is_empty() {
        found.push(format!(
            "standard output {}, expected empty",
            show(&output.stdout)
        ));
    }
    let expected = format!("{binary}: {name}\n");
    if output.stderr_text() != expected {
        found.push(format!(
            "standard error {:?}, expected {expected:?}",
            output.stderr_text()
        ));
    }
    if output.code != i32::from(code) {
        found.push(format!("exit status {}, expected {code}", output.code));
    }
    found
}

/// Hold the program behind `invoke` to the modeled response of every vector.
/// `invoke` runs `check` with the request on standard input. Returns how each
/// vector was compared and every disagreement, each naming its vector.
pub fn compare(cli: &Cli, invoke: &dyn Fn(&[u8]) -> Output) -> (Coverage, Vec<String>) {
    let mut coverage = Coverage::default();
    let mut mismatches = Vec::new();
    for (index, vector) in cli.vectors.iter().enumerate() {
        let output = invoke(&vector.request);
        let outcome = cli.classify(&vector.response);
        let found = if vector.request.len() > cli.request_maximum {
            coverage.oversized += 1;
            // The page's report for the same request is the modeled error.
            let mut found = match outcome {
                Some(outcome) if outcome.name == "error" => Vec::new(),
                _ => vec![format!(
                    "modeled response {} is not an error report",
                    show(&vector.response)
                )],
            };
            found.extend(refusal(
                &output,
                &cli.cargo_name,
                "InputLimit",
                cli.limit_exit_code(),
            ));
            found
        } else if cli.is_framed(&vector.request) {
            coverage.framed += 1;
            refusal(
                &output,
                &cli.cargo_name,
                "ReportUnclassified",
                cli.outcome("error").exit_code,
            )
        } else {
            coverage.bytes += 1;
            let mut found = Vec::new();
            if output.stdout != vector.response {
                found.push(format!(
                    "response {}, modeled {}",
                    show(&output.stdout),
                    show(&vector.response)
                ));
            }
            match outcome {
                Some(outcome) if output.code == i32::from(outcome.exit_code) => {}
                Some(outcome) => found.push(format!(
                    "exit status {}, expected {} for {}",
                    output.code, outcome.exit_code, outcome.name
                )),
                None => found.push("the modeled response has no declared outcome".to_owned()),
            }
            if !output.stderr.is_empty() {
                found.push(format!("standard error {}", show(&output.stderr)));
            }
            found
        };
        mismatches.extend(
            found
                .into_iter()
                .map(|what| format!("vector {index}: {what}")),
        );
    }
    (coverage, mismatches)
}

fn oversized(cli: &Cli) -> usize {
    cli.vectors
        .iter()
        .filter(|vector| vector.request.len() > cli.request_maximum)
        .count()
}

fn check_arguments(cli: &Cli) -> Vec<OsString> {
    vec![OsString::from(&cli.command)]
}

/// `PA-01`: the command line prints what verification holds the page to, for
/// every acceptance vector.
#[test]
fn the_page_and_the_command_line_agree_on_every_vector_pa_01() {
    let fixture = fixture();
    let cli = &fixture.cli;
    assert!(
        !cli.vectors.is_empty(),
        "the model declares no acceptance vector"
    );
    evidence::attestation(root(), &fixture.program).require(cli.vectors.len(), oversized(cli));
    let (coverage, mismatches) = compare(cli, &|request| {
        run(&fixture.program, &check_arguments(cli), request)
    });
    assert!(
        mismatches.is_empty(),
        "the command line differs from the response verification holds the page to:\n{}",
        mismatches.join("\n")
    );
    assert_eq!(
        coverage.total(),
        cli.vectors.len(),
        "every vector is compared"
    );
    assert!(coverage.bytes > 0, "no vector was compared by its bytes");
    assert!(
        coverage.oversized > 0,
        "no oversized vector was compared by its refusal"
    );
}

/// The planted defect: a program that prints one newline after the real
/// program's output and exits with its status. Run through `sh` rather than
/// executed, because a script written by one test and executed by another
/// thread races the kernel's refusal to execute a file open for writing.
fn with_trailing_newline(
    scratch: &Scratch,
    program: &Path,
    command: &str,
) -> impl Fn(&[u8]) -> Output {
    let script = scratch.file(
        "newline.sh",
        b"real=$1\nshift\n\"$real\" \"$@\"\nstatus=$?\nprintf '\\n'\nexit $status\n",
    );
    let program = program.to_owned();
    let command = command.to_owned();
    move |request| {
        let arguments = [
            script.clone().into_os_string(),
            program.clone().into_os_string(),
            OsString::from(&command),
        ];
        run(Path::new("/bin/sh"), &arguments, request)
    }
}

/// `PA-01`: a changed output is reported for every vector, so the comparison
/// cannot pass when the command line drifts from the page.
#[test]
fn a_trailing_newline_fails_the_comparison_pa_01() {
    let fixture = fixture();
    let cli = &fixture.cli;
    let scratch = Scratch::new();
    let wrapped = with_trailing_newline(&scratch, &fixture.program, &cli.command);
    let (coverage, mismatches) = compare(cli, &wrapped);
    assert_eq!(coverage.total(), cli.vectors.len());
    for index in 0..cli.vectors.len() {
        let prefix = format!("vector {index}: ");
        assert!(
            mismatches.iter().any(|found| found.starts_with(&prefix)),
            "vector {index} passed with a trailing newline added:\n{}",
            mismatches.join("\n")
        );
    }
}
