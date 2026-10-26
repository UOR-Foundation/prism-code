//! Project admission: the nine violations and the canonical form
//! (`AD-01` to `AD-10`).
//!
//! The model gates every statement through `Core.gate`, and
//! `Core.gateAdmitsExactly` with the five `Interaction` admission theorems says
//! a statement is admitted exactly when its admission check admits it. These
//! tests drive the generated program on requests the checks refuse and hold it
//! to the exact refusal.

use crate::fixture;
use crate::report::{accepted, expect, script, script_refusal, vector_of, vector_refusal};

/// `AD-01`: a second node of the same name, vector 6, whatever its kind.
#[test]
fn a_duplicate_node_is_refused_ad_01() {
    vector_refusal(
        &["node camera input 0", "node camera output 0"],
        3,
        "duplicate-node",
    );
    script_refusal(&["node a input 0", "node a input 0"], 3, "duplicate-node");
    script_refusal(
        &["node a input 0", "node b input 0", "node b operator 9"],
        4,
        "duplicate-node",
    );
}

/// `AD-02`: an unknown endpoint of an edge, vector 5, or an unknown pii node.
#[test]
fn an_unknown_node_is_refused_ad_02() {
    vector_refusal(
        &["node camera input 0", "edge camera detector"],
        3,
        "unknown-node",
    );
    script_refusal(&["node b operator 0", "edge a b"], 3, "unknown-node");
    script_refusal(&["pii ghost"], 2, "unknown-node");
    // Existence is checked before the self-edge rule.
    script_refusal(&["edge a a"], 2, "unknown-node");
}

/// `AD-03`: an edge from a node to itself, vector 18, ahead of the kind rules.
#[test]
fn a_self_edge_is_refused_ad_03() {
    vector_refusal(&["node a operator 0", "edge a a"], 3, "self-edge");
    script_refusal(&["node a output 0", "edge a a"], 3, "self-edge");
    script_refusal(&["node a input 0", "edge a a"], 3, "self-edge");
}

/// `AD-04`: the same edge twice, vector 19.
#[test]
fn a_duplicate_edge_is_refused_ad_04() {
    vector_refusal(
        &[
            "node a operator 0",
            "node b operator 0",
            "edge a b",
            "edge a b",
        ],
        5,
        "duplicate-edge",
    );
    // The reverse edge is a different edge.
    expect(
        &script(
            &fixture().cli,
            &[
                "node a operator 0",
                "node b operator 0",
                "edge a b",
                "edge b a",
            ],
        ),
        &accepted(
            &fixture().cli,
            &[
                "node a operator 0",
                "node b operator 0",
                "edge a b",
                "edge b a",
            ],
        ),
    );
}

/// `AD-05`: an edge leaving an output node, vector 20.
#[test]
fn an_edge_from_an_output_is_refused_ad_05() {
    vector_refusal(
        &["node a output 0", "node b operator 0", "edge a b"],
        4,
        "output-source",
    );
    script_refusal(
        &["node a output 0", "node b output 0", "edge a b"],
        4,
        "output-source",
    );
    script_refusal(
        &["node a output 0", "node b input 0", "edge a b"],
        4,
        "output-source",
    );
}

/// `AD-06`: an edge entering an input node, vector 21.
#[test]
fn an_edge_into_an_input_is_refused_ad_06() {
    vector_refusal(
        &["node a operator 0", "node b input 0", "edge a b"],
        4,
        "input-target",
    );
    script_refusal(
        &["node a input 0", "node b input 0", "edge a b"],
        4,
        "input-target",
    );
}

/// `AD-07`: a pii mark on a node that is not an input, vector 22.
#[test]
fn a_pii_mark_on_a_non_input_is_refused_ad_07() {
    vector_refusal(&["node a operator 0", "pii a"], 3, "not-an-input");
    script_refusal(&["node a encrypt 0", "pii a"], 3, "not-an-input");
    script_refusal(&["node a output 0", "pii a"], 3, "not-an-input");
}

/// `AD-08`: the same input marked twice, vector 23.
#[test]
fn a_duplicate_pii_mark_is_refused_ad_08() {
    vector_refusal(&["node a input 0", "pii a", "pii a"], 4, "duplicate-pii");
    script_refusal(
        &[
            "node a input 0",
            "node b input 0",
            "pii a",
            "pii b",
            "pii a",
        ],
        6,
        "duplicate-pii",
    );
}

/// `AD-09`: a second `policy memory`, vector 24, or `policy encrypt-pii`, vector 25.
#[test]
fn a_duplicate_policy_is_refused_ad_09() {
    vector_refusal(
        &["policy memory 1", "policy memory 2"],
        3,
        "duplicate-policy",
    );
    vector_refusal(
        &["policy encrypt-pii", "policy encrypt-pii"],
        3,
        "duplicate-policy",
    );
    // The same value twice is still a second policy.
    script_refusal(
        &["policy memory 1", "policy memory 1"],
        3,
        "duplicate-policy",
    );
    // The two policies are independent, so the repeat is what is refused.
    script_refusal(
        &["policy memory 1", "policy encrypt-pii", "policy memory 1"],
        4,
        "duplicate-policy",
    );
    script_refusal(
        &[
            "policy encrypt-pii",
            "policy memory 1",
            "policy encrypt-pii",
        ],
        4,
        "duplicate-policy",
    );
}

/// `AD-10`: the shuffled script with a blank line, vector 2, and the sorted
/// script, vector 3, give the same bytes, and byte order decides the order.
#[test]
fn the_canonical_form_is_sorted_and_independent_of_statement_order_ad_10() {
    let cli = &fixture().cli;
    let shuffled = &cli.vectors[1];
    let sorted = &cli.vectors[2];
    assert!(
        shuffled.request.windows(2).any(|pair| pair == b"\n\n"),
        "vector 2 carries a blank line"
    );
    assert_ne!(shuffled.request, sorted.request, "vectors 2 and 3 differ");
    assert_eq!(
        shuffled.response, sorted.response,
        "the model gives vectors 2 and 3 the same response"
    );
    // The sorted script is its own canonical form behind the status line.
    let status_end = sorted
        .response
        .iter()
        .position(|byte| *byte == b'\n')
        .expect("the response has a status line");
    assert_eq!(
        sorted.response[status_end + 1..],
        sorted.request[..],
        "vector 3 is already canonical"
    );
    expect(&shuffled.request, &shuffled.response);
    expect(&sorted.request, &sorted.response);

    // Names compare by bytes: uppercase before lowercase, a prefix before its
    // extension. Edges order by source, then target; policies come first.
    let request = script(
        cli,
        &[
            "node b operator 0",
            "policy encrypt-pii",
            "node B input 0",
            "node a input 0",
            "node ab operator 0",
            "edge b ab",
            "pii a",
            "edge B b",
            "pii B",
            "edge a b",
            "edge a ab",
            "policy memory 5",
        ],
    );
    assert!(
        vector_of(cli, &request).is_none(),
        "not an acceptance vector"
    );
    let expected = accepted(
        cli,
        &[
            "policy memory 5",
            "policy encrypt-pii",
            "node B input 0",
            "node a input 0",
            "node ab operator 0",
            "node b operator 0",
            "pii B",
            "pii a",
            "edge B b",
            "edge a ab",
            "edge a b",
            "edge b ab",
        ],
    );
    expect(&request, &expected);
}
