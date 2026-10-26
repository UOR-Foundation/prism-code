//! `--help` and `--version` (`CL-05`, `CL-06`, `CL-08`).

use crate::fixture;
use crate::program::{args, run};

fn first_token(line: &str) -> &str {
    line.split_whitespace().next().unwrap_or_default()
}

/// `CL-05`: the help carries every modeled section and the exit-code table.
#[test]
fn help_prints_the_modeled_text_and_exit_codes_cl_05() {
    let fixture = fixture();
    let cli = &fixture.cli;
    let output = run(&fixture.program, &args(&["--help"]), b"");
    assert_eq!(output.code, 0, "{}", output.stderr_text());
    assert!(output.stderr.is_empty(), "{}", output.stderr_text());
    let help = output.stdout_text();

    assert!(
        help.contains(cli.description.trim_end_matches('\n')),
        "the purpose is missing:\n{help}"
    );
    assert!(
        !cli.sections.is_empty(),
        "the model declares no help section"
    );
    for section in &cli.sections {
        let block = format!(
            "\n{}\n{}\n",
            section.title,
            section.body.trim_end_matches('\n')
        );
        assert!(
            help.contains(&block),
            "section {} is missing:\n{help}",
            section.title
        );
    }
    assert!(
        help.contains(&format!("at most {} of them", cli.request_maximum)),
        "the request bound is missing:\n{help}"
    );
    for example in &cli.examples {
        assert!(
            help.contains(&format!("\n    {}\n", example.name)),
            "example {} is not listed:\n{help}",
            example.name
        );
    }

    for outcome in &cli.outcomes {
        let row = format!("report begins with {:?}", outcome.prefix);
        let line = help
            .lines()
            .find(|line| line.contains(&row))
            .unwrap_or_else(|| panic!("no exit-code row for {}:\n{help}", outcome.name));
        assert_eq!(first_token(line), outcome.exit_code.to_string(), "{line}");
    }
    for (meaning, code) in [
        ("invalid command line", cli.usage_code),
        ("input or output failure", cli.io_code),
    ] {
        let line = help
            .lines()
            .find(|line| line.contains(meaning))
            .unwrap_or_else(|| panic!("no exit-code row for {meaning}:\n{help}"));
        assert_eq!(first_token(line), code.to_string(), "{line}");
    }
}

/// `CL-06`: the version is the model's name and version.
#[test]
fn version_prints_the_modeled_name_and_version_cl_06() {
    let fixture = fixture();
    let cli = &fixture.cli;
    let output = run(&fixture.program, &args(&["--version"]), b"");
    assert_eq!(output.code, 0, "{}", output.stderr_text());
    assert!(output.stderr.is_empty(), "{}", output.stderr_text());
    assert_eq!(
        output.stdout_text(),
        format!("{} {}\n", cli.cargo_name, cli.cargo_version)
    );
    assert_eq!(
        cli.cargo_version, cli.product_version,
        "the program and the system model report different versions"
    );
    // The value #18 asks of the v0.1.0 milestone.
    assert_eq!(
        (cli.cargo_name.as_str(), cli.cargo_version.as_str()),
        ("dataflow-policy", "0.1.0")
    );
}

/// `CL-08`: nothing the page teaches is missing from the command line.
#[test]
fn the_help_carries_every_workbench_string_cl_08() {
    let fixture = fixture();
    let (cli, guidance) = (&fixture.cli, &fixture.guidance);
    let help = run(&fixture.program, &args(&["--help"]), b"");
    assert_eq!(help.code, 0, "{}", help.stderr_text());
    let mut texts = String::new();
    for example in &cli.examples {
        let printed = run(&fixture.program, &args(&["examples", &example.name]), b"");
        assert_eq!(
            printed.code,
            0,
            "{}: {}",
            example.name,
            printed.stderr_text()
        );
        assert_eq!(printed.stdout_text(), example.text, "{}", example.name);
        texts.push_str(&printed.stdout_text());
    }
    let help = help.stdout_text();

    assert!(
        !guidance.forms.is_empty(),
        "the workbench states no statement form"
    );
    assert!(
        !guidance.statuses.is_empty(),
        "the workbench states no status text"
    );
    assert!(
        !guidance.recoveries.is_empty(),
        "the workbench states no recovery text"
    );
    for (usage, meaning) in &guidance.forms {
        let entry = format!("    {usage}\n        {meaning}\n");
        assert!(
            help.contains(&entry),
            "statement form {usage} is missing:\n{help}"
        );
    }
    for (name, text) in &guidance.statuses {
        let entry = format!("    {name}\n        {text}\n");
        assert!(
            help.contains(&entry),
            "status text {name} is missing:\n{help}"
        );
    }
    for recovery in &guidance.recoveries {
        let entry = format!(
            "    {}\n        {}\n        Fix: {}\n",
            recovery.code, recovery.explanation, recovery.fix
        );
        assert!(
            help.contains(&entry),
            "recovery text for {} is missing:\n{help}",
            recovery.code
        );
    }
    for example in &cli.examples {
        assert!(
            texts.contains(&example.text),
            "example {} is not printed",
            example.name
        );
    }
}
