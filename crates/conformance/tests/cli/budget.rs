//! Quality budgets (`QB-01` to `QB-03`).
//!
//! `src/Quality.lex.tex` gives each performance-efficiency requirement a
//! measure and a threshold. The build these tests run against is the one
//! `prismpm build` published for the program under test, so the sizes are the
//! sizes of what ships. Each measured value is printed beside its budget, as a
//! report, and compared with the threshold the model states; the measurement is
//! an `open` claim and only the comparison is asserted.
//!
//! Budgets come from the model and nowhere else: changing a threshold in the
//! model changes what the build is held to.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::fixture;
use crate::model::{self, Budget};
use crate::program::{run, Output};
use crate::root;

/// The build directory that produced `program`: `.prism/build/<id>/`, three
/// levels above `cli/bin/<target>/<binary>`.
fn build_of(program: &Path) -> PathBuf {
    let build = program
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| panic!("{} is not under a build", program.display()));
    assert!(
        build.join("view").join("browser").is_dir(),
        "{} has no view/browser: the program was not published by prismpm build",
        build.display()
    );
    build.to_owned()
}

fn size(path: &Path) -> u64 {
    std::fs::metadata(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .len()
}

/// Every file of the browser closure. The SDK fixes it at six files; a missing
/// or extra file is a different closure and is not measured.
fn closure(build: &Path) -> Vec<PathBuf> {
    let dir = build.join("view").join("browser");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
                .path()
        })
        .collect();
    files.sort();
    for file in &files {
        assert!(file.is_file(), "{} is not a file", file.display());
    }
    files
}

/// The Core-Wasm guest: the one `.wasm` file under `core-wasm/`.
fn guest(build: &Path) -> PathBuf {
    let dir = build.join("core-wasm");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
                .path()
        })
        .filter(|path| path.extension().is_some_and(|e| e == "wasm"))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "{} holds {} guests",
        dir.display(),
        found.len()
    );
    found.remove(0)
}

fn report(budget: &Budget, measured: u64) {
    // Written past the test harness's capture, so the build log of every
    // `just vv` carries each budget beside its measured value.
    let line = format!(
        "budget {} ({}): measured {measured} {}, budget at most {} {} ({})\n",
        budget.measure, budget.name, budget.unit, budget.threshold, budget.unit, budget.basis
    );
    let _ = std::io::stderr().write_all(line.as_bytes());
}

fn within(budget: &Budget, measured: u64) {
    report(budget, measured);
    assert!(
        measured <= budget.threshold,
        "{} is {measured} {}, over its budget of {} {}",
        budget.name,
        budget.unit,
        budget.threshold,
        budget.unit
    );
}

/// `QB-01`: the Core-Wasm guest is within its byte budget.
#[test]
fn the_guest_is_within_its_size_budget_qb_01() {
    let budgets = model::load_budgets(root());
    let build = build_of(&fixture().program);
    within(&budgets.guest, size(&guest(&build)));
}

/// `QB-01`, planted: a guest one byte over the budget is refused.
#[test]
fn a_guest_over_its_budget_is_refused_qb_01() {
    let budgets = model::load_budgets(root());
    let over = budgets.guest.threshold + 1;
    let refused = std::panic::catch_unwind(|| within(&budgets.guest, over));
    assert!(refused.is_err(), "a {over}-byte guest passed its budget");
}

/// `QB-02`: the browser closure is within its byte budget.
#[test]
fn the_browser_closure_is_within_its_size_budget_qb_02() {
    let budgets = model::load_budgets(root());
    let build = build_of(&fixture().program);
    let files = closure(&build);
    assert_eq!(
        files.len(),
        6,
        "the browser closure is six files: {files:?}"
    );
    within(&budgets.closure, files.iter().map(|file| size(file)).sum());
}

/// The largest request the model declares: vector 12, the 4096-byte script.
fn largest(cli: &model::Cli) -> &model::Vector {
    let vector = cli
        .vectors
        .iter()
        .filter(|vector| vector.request.len() <= cli.request_maximum)
        .max_by_key(|vector| vector.request.len())
        .expect("the model declares no vector within the request maximum");
    assert_eq!(vector.request.len(), cli.request_maximum);
    vector
}

/// The median of `runs` checks of `request`, each held to the modeled response
/// so that a fast wrong answer is not a fast check.
fn median(program: &Path, cli: &model::Cli, vector: &model::Vector, runs: usize) -> Duration {
    let arguments = [std::ffi::OsString::from(&cli.command)];
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let started = Instant::now();
            let output: Output = run(program, &arguments, &vector.request);
            let elapsed = started.elapsed();
            assert_eq!(
                output.stdout, vector.response,
                "the timed check gave another report"
            );
            elapsed
        })
        .collect();
    times.sort();
    times[runs / 2]
}

/// `QB-03`: checking the largest request with the command line, process start
/// included, is within its time budget. Timed once per `just vv` on vector 12,
/// as the median of 11 runs.
#[test]
fn the_command_line_checks_the_largest_request_within_its_time_budget_qb_03() {
    let fixture = fixture();
    let budgets = model::load_budgets(root());
    let vector = largest(&fixture.cli);
    let measured = median(&fixture.program, &fixture.cli, vector, 11);
    within(
        &budgets.cli_latency,
        u64::try_from(measured.as_millis()).expect("milliseconds fit in u64"),
    );
}
