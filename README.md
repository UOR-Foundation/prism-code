# prism-code

prism-code checks a Prism dataflow project script against its declared memory
and PII-encryption policies, and emits the project's canonical form. It is a
Prism application: its behavior is authored once in LexLean (`src/*.lex.tex`)
and PrismPM generates the verified browser application (GitHub Pages) and the
command-line interface from that single model. No application behavior is
written by hand outside the model.

## Status

Development, version 0.1.0. The authoritative model is committed; the SDK
binding, generated artifacts, CLI, and Pages deployment are not yet in place.
The work is tracked in the prism-code GitHub project and its milestones.

The model is LexLean language 1.2 notation, built with LexLean v0.4 and locked
in `lexlean.lock` (`lexlean/lock/2`). Its product identifiers (`prism-code`,
`PrismCode`, the `prism-code/1` script header, and the repository and Pages
URLs) are bound by the application model digest in `System` and by the lock, so
they change only through the LexLean and PrismPM toolchain, not by hand.

## What it does

A request is UTF-8 text of at most 4096 bytes. The first line is the header
`prism-code/1`; each further non-empty line is one statement:

| Statement | Meaning |
| --- | --- |
| `node NAME KIND MEMORY` | a node of kind `input`, `operator`, `encrypt`, or `output` with a memory requirement in bytes |
| `edge SOURCE TARGET` | a dataflow edge between two declared nodes |
| `pii NAME` | marks an input node as carrying personal data |
| `policy memory LIMIT` | no node may require more than `LIMIT` bytes |
| `policy encrypt-pii` | personal data must pass through an `encrypt` node before any `output` |

The response is one of:

- `status accepted` followed by the canonical, sorted project;
- `status rejected` followed by every policy finding (`finding memory ...`,
  `finding plaintext-pii ...`) and the canonical project; or
- `status error` with the first failing line and its typed code
  (`missing-header`, `unknown-command`, `wrong-arity`, `empty-field`,
  `unknown-kind`, `invalid-number`, `duplicate-node`, `unknown-node`,
  `self-edge`, `duplicate-edge`, `output-source`, `input-target`,
  `not-an-input`, `duplicate-pii`, `duplicate-policy`, `line-overflow`) or
  request code (`empty`, `too-large`, `invalid-utf8`, `too-many-lines`), or
  `response too-large` when the report would exceed 16384 bytes.

The model proves, among other things, that a run is accepted only if the
project is compliant (`Engine.completedRunAcceptedOnlyIfCompliant`) and that a
failed run is never accepted (`Engine.failedRunNeverAccepted`).

## Command line

The command-line interface is declared in the model, not written by hand. The
application record in `Specification` uses the `prismpm/text-application/3`
profile (`Foundation.View.Text.V3.Model`), whose `cli` field holds the command,
description, help sections, named examples and exit-code map. PrismPM
generates the program from that record:

```
prism-code check [FILE]    read the script from FILE, or from stdin when FILE
                           is omitted or `-`; print the report unchanged
prism-code examples [NAME] print a named example script
prism-code --help
prism-code --version
```

A request over 4096 bytes is refused before dispatch. Exit codes come from the
model: `status accepted` 0, `status rejected` 1, `status error` 2, usage error
64, input or output error 74.

## Repository layout

| Path | Owner |
| --- | --- |
| `src/` | authoritative LexLean model modules (`System.lex.tex` is the entrypoint) |
| `language/prism.arch/` | the architecture lexicon the model's glossary uses |
| `lexlean.toml`, `lexlean.lock` | LexLean project and its locked package closure |
| `prismpm.toml` | PrismPM project; generated output goes to `.prism/` |
| `lakefile.toml`, `lean-toolchain`, `lake-manifest.json` | Lean workspace bound by `lexlean.lock` |
| `model/`, `features/suites/`, `CONFORMANCE.md` | the repository's claim register (template contract) |
| `crates/`, `xtask/`, `bootstrap/` | repository gates inherited from the UOR template |

| Module | Content |
| --- | --- |
| `Core` | nodes, edges, policies, violations, canonical form |
| `Interaction` | script statements, parsing, admission of each statement |
| `Compliance` | memory and PII-propagation findings |
| `Engine` | line runner, certification, outcome rendering |
| `Specification` | request dispatch and the text application with its acceptance vectors |
| `Workbench` | page guidance: statement forms, examples drawn from the acceptance vectors, status texts and recovery text for every error and finding code |
| `Foundation.View.Text.V1.Model` | the text view contract |
| `Production.Core`, `Production.BrowserSystem` | system model types and manifest validation |
| `System` | the prism-code system model and its GitHub Pages target |
| `Architecture` | the ISO/IEC/IEEE 42010 architecture description, checked against `System` |
| `Design` | the OPM (ISO/PAS 19450) process model and the UML component, sequence and state machine views of the architecture, each element checked against the architecture and system models |
| `Foundation.Arch` | the architecture and design-view vocabulary (copy of the PrismPM stdlib module at PrismPM#234) |

## Toolchain

prism-code is built with LexLean and the PrismPM SDK, used only through the
digest-pinned SDK image in `prismpm.lock`. That lock and `template.lock` are
written when the repository binds a public PrismPM SDK release; until then
`just vv` and the bootstrap workflow cannot pass. See
[TEMPLATE-CONTRACT.md](TEMPLATE-CONTRACT.md), [AGENTS.md](AGENTS.md), and
[VERIFICATION.md](VERIFICATION.md).

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
