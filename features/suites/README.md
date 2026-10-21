# Suites

One Gherkin file per suite, one scenario per conformance ID (R3).

A scenario is tagged with its ID and its honesty level:

```gherkin
Feature: <suite name>

  <what the suite is about, in a sentence.>

  @CX-01 @build
  Scenario: <the statement, copied from model/ids.toml>
    Given <the fixture>
    When the suite exercises CX-01
    Then <the assertion, in the register's words>
```

The `suite` field of a row in `model/ids.toml` names the file its scenario
lives in, and `just bdd` fails if an ID has no scenario, a scenario has no ID,
or an ID has no test whose name ends in it lowercased with underscores.

Empty. The first capability this repository builds starts with a row in
`model/ids.toml`, then a scenario here, then a failing test --- in that order,
because the order is the discipline (`AGENTS.md`).

## Public error codes (R5)

Every public failure the application can emit, and the acceptance vector in
`Specification.application.acceptanceVectors` that produces it. Vectors are
numbered from 1 in list order. Each vector is executed by
`dispatch request == response` against the generated Lean modules.

| Code | Where it is raised | Vector |
| --- | --- | --- |
| `request empty` | `Specification.dispatch` | 11 |
| `request invalid-utf8` | `Specification.dispatch` | 10 |
| `request too-large` | `Specification.dispatch` | 13 |
| `line N missing-header` | `Specification.dispatchText` | 9 |
| `line N unknown-command` | `Interaction.parseFields`, `parsePolicyFields` | 14 |
| `line N wrong-arity` | `Interaction.parseLine` and the `parse*Fields` family | 15 |
| `line N empty-field` | `Interaction.parseLine` | 8 |
| `line N unknown-kind` | `Interaction.parseNode` | 7 |
| `line N invalid-number` | `Interaction.parseNode` (node memory) | 16 |
| `line N invalid-number` | `Interaction.parseMemoryPolicy` (policy limit) | 17 |
| `line N duplicate-node` | `Interaction.admitNode` | 6 |
| `line N unknown-node` | `Interaction.admitEdge`, `admitPii` | 5 |
| `line N self-edge` | `Interaction.admitEdge` | 18 |
| `line N duplicate-edge` | `Interaction.admitEdge` | 19 |
| `line N output-source` | `Interaction.admitEdge` | 20 |
| `line N input-target` | `Interaction.admitEdge` | 21 |
| `line N not-an-input` | `Interaction.admitPii` | 22 |
| `line N duplicate-pii` | `Interaction.admitPii` | 23 |
| `line N duplicate-policy` | `Interaction.admitMemoryPolicy` (second `policy memory`) | 24 |
| `line N duplicate-policy` | `Interaction.admitPiiPolicy` (second `policy encrypt-pii`) | 25 |
| `request too-many-lines` | `Specification.dispatchText` | none: unreachable, see below |
| `line N line-overflow` | `Engine.runLines` | none: unreachable, see below |
| `response too-large` | `Specification.respond` | none: unreachable, see below |

### Codes with no vector

A request is at most 4096 bytes (`requestMaximum`), so:

- `request too-many-lines`: `dispatchText` allows 4097 lines. 4096 bytes hold
  at most 4096 newline bytes, hence at most 4097 lines, which is the limit.
  Vector 12 is the boundary case (4096 bytes, 4097 lines, accepted).
- `line-overflow`: `Engine.runLines` starts counting at line 1 and fails only
  when the `UInt32` counter would pass 2^32 - 1, which needs more than 2^32
  lines. A request has at most 4097.
- `response too-large`: a response is the status line, the findings (at most
  two per node) and the canonical form (one line per accepted line, no longer
  than the request line it came from, plus the 13-byte header). The densest
  request found (an input labelled as PII feeding 2-letter output nodes whose
  memory exceeds the limit, 4091 bytes) produces a 12302-byte response, below
  the 16384-byte `responseMaximum`. The growth factor is about 3, so the
  limit is not reachable from 4096 bytes.

These three arguments are not machine-checked. The proof language
(`SPEC.md` section 16) has no lemma about `split_exact`, `utf8_encode` length or
`UInt32` `checked_add`, so the bounds cannot yet be stated as theorems over the
primitives. The branches stay in the model, and the table row is the open
obligation: a theorem or a vector must replace it when the lemmas exist.
