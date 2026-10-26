//! Policy compliance: the two findings, their order, the acceptance guarantee
//! and the absence of findings without a policy (`CP-01` to `CP-05`).
//!
//! `Compliance.findings` computes the findings, `Engine.certify` accepts a
//! project exactly when there are none, and the generated program prints what
//! the model computes.

use crate::fixture;
use crate::report::{
    accepted, assert_modeled, expect, expect_script, expect_vector, failed, rejected, script,
    script_refusal, vector_of,
};

/// `CP-01`: a node using more than the limit is a finding, one at the limit is
/// not, and every finding names the node, its use and the limit.
#[test]
fn a_node_over_the_memory_limit_is_a_finding_cp_01() {
    let cli = &fixture().cli;
    let lines = ["policy memory 10", "node a operator 11"];
    expect_script(
        &lines,
        &rejected(
            cli,
            &["finding memory a 11 exceeds 10"],
            &["policy memory 10", "node a operator 11"],
        ),
    );
    expect_script(
        &["policy memory 10", "node a operator 10"],
        &accepted(cli, &["policy memory 10", "node a operator 10"]),
    );
    // Every kind of node is measured, and each over the limit is its own finding.
    expect_script(
        &["policy memory 0", "node b output 2", "node a input 1"],
        &rejected(
            cli,
            &[
                "finding memory a 1 exceeds 0",
                "finding memory b 2 exceeds 0",
            ],
            &["policy memory 0", "node a input 1", "node b output 2"],
        ),
    );
    // Vector 4: the operator at 2000000 against a limit of 1000000.
    let vector = &cli.vectors[3];
    assert!(
        String::from_utf8_lossy(&vector.response)
            .contains("finding memory detector 2000000 exceeds 1000000"),
        "vector 4 reports the memory finding"
    );
    expect(&vector.request, &vector.response);
}

/// `CP-02`: PII reaching an output without passing an encrypt node is a
/// finding, and the vector with two outputs reports exactly one.
#[test]
fn pii_reaching_an_output_in_plaintext_is_a_finding_cp_02() {
    let cli = &fixture().cli;
    expect_script(
        &[
            "policy encrypt-pii",
            "node a input 0",
            "node b output 0",
            "pii a",
            "edge a b",
        ],
        &rejected(
            cli,
            &["finding plaintext-pii b"],
            &[
                "policy encrypt-pii",
                "node a input 0",
                "node b output 0",
                "pii a",
                "edge a b",
            ],
        ),
    );
    // The label follows edges through operators and stops at an encrypt node.
    expect_script(
        &[
            "policy encrypt-pii",
            "node a input 0",
            "node m operator 0",
            "node n operator 0",
            "node o output 0",
            "pii a",
            "edge a m",
            "edge m n",
            "edge n o",
        ],
        &rejected(
            cli,
            &["finding plaintext-pii o"],
            &[
                "policy encrypt-pii",
                "node a input 0",
                "node m operator 0",
                "node n operator 0",
                "node o output 0",
                "pii a",
                "edge a m",
                "edge m n",
                "edge n o",
            ],
        ),
    );
    // Vector 1, whose only path to the output is through the encrypt node, is accepted.
    let encrypted = &cli.vectors[0];
    assert_modeled(&encrypted.request, &encrypted.response);
    expect(&encrypted.request, &encrypted.response);
    assert!(
        encrypted
            .response
            .starts_with(cli.outcome("accepted").prefix.as_bytes()),
        "vector 1 is accepted"
    );

    // PII reaches two outputs, one through encrypt and one not: one finding.
    let two = [
        "policy encrypt-pii",
        "node camera input 0",
        "node vault encrypt 1000",
        "node safe output 0",
        "node leak output 0",
        "pii camera",
        "edge camera vault",
        "edge vault safe",
        "edge camera leak",
    ];
    let request = script(cli, &two);
    let vector = vector_of(cli, &request)
        .unwrap_or_else(|| panic!("no acceptance vector has two outputs, one through encrypt"));
    let response = String::from_utf8_lossy(&vector.response).into_owned();
    let found: Vec<&str> = response
        .lines()
        .filter(|line| line.starts_with("finding plaintext-pii"))
        .collect();
    assert_eq!(found, ["finding plaintext-pii leak"], "{response}");
    expect_vector(
        &two,
        &rejected(
            cli,
            &["finding plaintext-pii leak"],
            &[
                "policy encrypt-pii",
                "node camera input 0",
                "node leak output 0",
                "node safe output 0",
                "node vault encrypt 1000",
                "pii camera",
                "edge camera leak",
                "edge camera vault",
                "edge vault safe",
            ],
        ),
    );
}

/// `CP-03`: memory findings come before plaintext-pii findings, and each kind
/// is in name order whatever the order of the statements.
#[test]
fn findings_are_ordered_memory_first_then_by_name_cp_03() {
    let cli = &fixture().cli;
    let request_lines = [
        "policy memory 1",
        "policy encrypt-pii",
        "node src input 0",
        "node z operator 5",
        "node y operator 5",
        "node q output 0",
        "node p output 0",
        "pii src",
        "edge src z",
        "edge src y",
        "edge z q",
        "edge y p",
    ];
    expect_script(
        &request_lines,
        &rejected(
            cli,
            &[
                "finding memory y 5 exceeds 1",
                "finding memory z 5 exceeds 1",
                "finding plaintext-pii p",
                "finding plaintext-pii q",
            ],
            &[
                "policy memory 1",
                "policy encrypt-pii",
                "node p output 0",
                "node q output 0",
                "node src input 0",
                "node y operator 5",
                "node z operator 5",
                "pii src",
                "edge src y",
                "edge src z",
                "edge y p",
                "edge z q",
            ],
        ),
    );
    // Vector 4 has one finding of each kind, the memory one first.
    let vector = &cli.vectors[3];
    let response = String::from_utf8_lossy(&vector.response).into_owned();
    let findings: Vec<&str> = response
        .lines()
        .filter(|line| line.starts_with("finding "))
        .collect();
    assert_eq!(
        findings,
        [
            "finding memory detector 2000000 exceeds 1000000",
            "finding plaintext-pii screen"
        ],
        "{response}"
    );
    expect(&vector.request, &vector.response);
}

/// `CP-04`: a project is accepted exactly when it has no finding, and a failed
/// run is never accepted.
#[test]
fn a_project_is_accepted_only_if_it_is_compliant_cp_04() {
    let cli = &fixture().cli;
    let mut seen = [0usize; 3];
    for (index, vector) in cli.vectors.iter().enumerate() {
        if vector.request.len() > cli.request_maximum || cli.is_framed(&vector.request) {
            continue;
        }
        let output = crate::report::check(&vector.request);
        let report = String::from_utf8_lossy(&output.stdout).into_owned();
        let findings = report
            .lines()
            .filter(|line| line.starts_with("finding "))
            .count();
        let outcome = cli
            .classify(&output.stdout)
            .unwrap_or_else(|| panic!("vector {}: unclassified report {report:?}", index + 1));
        match outcome.name.as_str() {
            "accepted" => {
                assert_eq!(findings, 0, "vector {} accepted with findings", index + 1);
                seen[0] += 1;
            }
            "rejected" => {
                assert!(findings > 0, "vector {} rejected with none", index + 1);
                seen[1] += 1;
            }
            "error" => {
                assert_eq!(findings, 0, "vector {} failed with findings", index + 1);
                seen[2] += 1;
            }
            other => panic!("vector {}: outcome {other}", index + 1),
        }
        assert_eq!(
            output.code,
            i32::from(outcome.exit_code),
            "vector {}",
            index + 1
        );
    }
    assert!(
        seen.iter().all(|count| *count > 0),
        "outcomes seen {seen:?}"
    );

    // A project that breaks a policy and then fails to parse is an error, not a
    // rejection: the run failed, so nothing is accepted or certified.
    script_refusal(
        &["policy memory 1", "node a operator 5", "bogus x"],
        4,
        "unknown-command",
    );
    expect_script(
        &["policy memory 1", "node a operator 5", "node a input 0"],
        &failed(cli, 4, "duplicate-node"),
    );
}

/// `CP-05`: without a policy there is no finding, so the project is accepted.
#[test]
fn a_policy_that_is_absent_produces_no_finding_cp_05() {
    let cli = &fixture().cli;
    // A node far over any limit, and PII reaching an output, with no policy.
    let both = [
        "node a input 9",
        "node m operator 18446744073709551615",
        "node b output 0",
        "pii a",
        "edge a m",
        "edge m b",
    ];
    expect_script(
        &both,
        &accepted(
            cli,
            &[
                "node a input 9",
                "node b output 0",
                "node m operator 18446744073709551615",
                "pii a",
                "edge a m",
                "edge m b",
            ],
        ),
    );
    // The memory policy alone does not check PII.
    expect_script(
        &[
            "policy memory 20",
            "node a input 9",
            "node b output 0",
            "pii a",
            "edge a b",
        ],
        &accepted(
            cli,
            &[
                "policy memory 20",
                "node a input 9",
                "node b output 0",
                "pii a",
                "edge a b",
            ],
        ),
    );
    // The encryption policy alone does not check memory.
    expect_script(
        &["policy encrypt-pii", "node m operator 18446744073709551615"],
        &accepted(
            cli,
            &["policy encrypt-pii", "node m operator 18446744073709551615"],
        ),
    );
    // With the policy and no node marked as PII there is nothing to leak.
    expect_script(
        &[
            "policy encrypt-pii",
            "node a input 0",
            "node b output 0",
            "edge a b",
        ],
        &accepted(
            cli,
            &[
                "policy encrypt-pii",
                "node a input 0",
                "node b output 0",
                "edge a b",
            ],
        ),
    );
}
