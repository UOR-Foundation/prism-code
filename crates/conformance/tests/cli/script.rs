//! Script parsing: the five statement forms and the five parse errors
//! (`SC-01` to `SC-10`).
//!
//! Every test runs the generated program on a request and compares the printed
//! bytes with the exact report. A request that an acceptance vector carries is
//! tied to that vector, so the expected response is the model's.

use crate::fixture;
use crate::report::{accepted, expect_script, script_refusal, vector_refusal};

/// A script made only of canonical lines is accepted and printed back as it is.
fn echoed(lines: &[&str]) {
    expect_script(lines, &accepted(&fixture().cli, lines));
}

/// `SC-01`: `node NAME KIND MEMORY`, for each of the four kinds.
#[test]
fn a_node_statement_adds_a_node_of_each_kind_sc_01() {
    echoed(&["node a input 3"]);
    echoed(&["node b operator 4"]);
    echoed(&["node c encrypt 5"]);
    echoed(&["node d output 6"]);
    echoed(&["node z output 0"]);
}

/// `SC-02`: `edge SOURCE TARGET`.
#[test]
fn an_edge_statement_connects_two_declared_nodes_sc_02() {
    echoed(&["node a input 0", "node b operator 0", "edge a b"]);
    echoed(&["node a input 0", "node b output 0", "edge a b"]);
}

/// `SC-03`: `pii NAME` marks an input and adds no finding without the policy.
#[test]
fn a_pii_statement_marks_an_input_node_sc_03() {
    echoed(&["node a input 0", "pii a"]);
    echoed(&["node a input 0", "node b output 0", "pii a", "edge a b"]);
}

/// `SC-04`: `policy memory LIMIT`, where a node at the limit does not exceed it.
#[test]
fn a_memory_policy_statement_sets_the_limit_sc_04() {
    echoed(&["policy memory 10", "node a operator 10"]);
    echoed(&["policy memory 0", "node a operator 0"]);
}

/// `SC-05`: `policy encrypt-pii` takes no operand.
#[test]
fn an_encrypt_pii_policy_statement_enables_the_policy_sc_05() {
    echoed(&["policy encrypt-pii", "node a input 0"]);
}

/// `SC-06`: an unknown command, vector 14, and the blank line that is counted.
#[test]
fn an_unknown_command_is_refused_with_its_line_sc_06() {
    vector_refusal(&["bogus x"], 2, "unknown-command");
    script_refusal(&["bogus"], 2, "unknown-command");
    script_refusal(&["Node a input 0"], 2, "unknown-command");
    script_refusal(&["policy bogus"], 2, "unknown-command");
    script_refusal(&["policy memory-limit 1"], 2, "unknown-command");
    // A blank line is ignored but still counts toward the line number.
    script_refusal(&["", "bogus x"], 3, "unknown-command");
}

/// `SC-07`: the wrong number of operands for each statement, vector 15 first.
#[test]
fn a_statement_with_the_wrong_arity_is_refused_sc_07() {
    vector_refusal(&["node a input"], 2, "wrong-arity");
    for line in [
        "node",
        "node a",
        "node a input 0 x",
        "node a input 0 x y",
        "edge",
        "edge a",
        "edge a b c",
        "pii",
        "pii a b",
        "policy",
        "policy memory",
        "policy memory 1 2",
        "policy encrypt-pii x",
    ] {
        script_refusal(&[line], 2, "wrong-arity");
    }
}

/// `SC-08`: an empty field, vector 8, from a doubled, leading or trailing space.
#[test]
fn an_empty_field_is_refused_sc_08() {
    vector_refusal(&["node camera  input 0"], 2, "empty-field");
    for line in [" node a input 0", "pii a ", "edge  a b", " "] {
        script_refusal(&[line], 2, "empty-field");
    }
}

/// `SC-09`: a node kind that is not one of the four, vector 7.
#[test]
fn an_unknown_node_kind_is_refused_sc_09() {
    vector_refusal(&["node camera sensor 0"], 2, "unknown-kind");
    script_refusal(&["node a Input 0"], 2, "unknown-kind");
    script_refusal(&["node a inputs 0"], 2, "unknown-kind");
    // The kind is read before the number.
    script_refusal(&["node a sensor x"], 2, "unknown-kind");
}

/// `SC-10`: a memory or limit that is not an unsigned 64-bit decimal.
#[test]
fn an_invalid_number_is_refused_sc_10() {
    vector_refusal(&["node a input x"], 2, "invalid-number");
    vector_refusal(&["policy memory x"], 2, "invalid-number");
    for line in [
        "node a input -1",
        "node a input 1.5",
        "node a input 18446744073709551616",
        "policy memory -1",
        "policy memory 18446744073709551616",
    ] {
        script_refusal(&[line], 2, "invalid-number");
    }
}
