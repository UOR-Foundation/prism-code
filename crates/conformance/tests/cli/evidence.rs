//! The SDK's verification evidence for the program under test.
//!
//! `prismpm verify` executes every acceptance vector in Core-Wasm and in the
//! portable browser view, holds both to the modeled response, and runs the
//! built program over the same vectors. It publishes the result under
//! `.prism/verified/<attestation>/application-acceptance.json` and does not
//! record response bytes, only that each run matched the model. The parity
//! test therefore needs two things from it: proof that the evidence is about
//! this program, which is the program's SHA-256 recorded in the evidence, and
//! the counts that show every vector was in scope.

use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

/// A verification record that attests the program under test.
pub struct Attestation {
    /// The `application-acceptance.json` that was read.
    pub path: PathBuf,
    /// The parsed document.
    pub document: Value,
}

fn sha256_hex(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    format!("{:x}", Sha256::digest(&bytes))
}

fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()))
}

/// Whether `document` names a program for its executed target with this digest.
fn attests(document: &Value, digest: &str) -> bool {
    let line = &document["command_line"];
    let Some(target) = line["executed_target"].as_str() else {
        return false;
    };
    line["programs"].as_array().is_some_and(|programs| {
        programs.iter().any(|program| {
            program["target"].as_str() == Some(target) && program["sha256"].as_str() == Some(digest)
        })
    })
}

/// The verification evidence for `program`, found under `root/.prism/verified`.
///
/// Evidence left by an earlier build attests a different program and is not
/// matched. Absence is a failure, not a skip: a parity claim about a program
/// the SDK never verified is not a claim.
pub fn attestation(root: &Path, program: &Path) -> Attestation {
    let digest = sha256_hex(program);
    let verified = root.join(".prism").join("verified");
    let entries = std::fs::read_dir(&verified).unwrap_or_else(|error| {
        panic!(
            "no verification evidence: {}: {error}. Run `prismpm verify` first",
            verified.display()
        )
    });
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", verified.display()));
        let path = entry.path().join("application-acceptance.json");
        if !path.is_file() {
            continue;
        }
        let document = read_json(&path);
        if attests(&document, &digest) {
            found.push(Attestation { path, document });
        }
    }
    match found.len() {
        1 => found.remove(0),
        0 => panic!(
            "no evidence under {} attests {} (sha256 {digest}). Run `prismpm verify` after \
             `prismpm build`",
            verified.display(),
            program.display()
        ),
        count => panic!(
            "{count} verification records under {} attest {}; clean `.prism` and verify again",
            verified.display(),
            program.display()
        ),
    }
}

/// The SDK's summary of the browser's dispatch-to-render time for one
/// acceptance vector, over its repetitions.
#[derive(Debug, PartialEq, Eq)]
pub struct BrowserTiming {
    /// The median over the repetitions, in whole microseconds.
    pub median_microseconds: u64,
    /// The slowest repetition, in whole microseconds.
    pub maximum_microseconds: u64,
    /// How many times the vector was dispatched.
    pub repetitions: u64,
    /// The browser engine that was timed.
    pub engine: String,
    /// The version of that engine.
    pub engine_version: String,
}

impl Attestation {
    /// The browser timing `prismpm verify` recorded for acceptance vector
    /// `index`. A record without it is a failure: the budget it serves would
    /// otherwise be held to nothing.
    pub fn browser_timing(&self, index: usize) -> BrowserTiming {
        let name = self.path.display();
        let rows = self.document["browser_timing"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: no browser_timing; the SDK recorded none"));
        let mut matching = rows
            .iter()
            .filter(|row| row["vector_index"].as_u64() == u64::try_from(index).ok());
        let row = matching
            .next()
            .unwrap_or_else(|| panic!("{name}: browser_timing has no row for vector {index}"));
        assert!(
            matching.next().is_none(),
            "{name}: browser_timing has two rows for vector {index}"
        );
        let number = |key: &str| {
            row[key]
                .as_u64()
                .unwrap_or_else(|| panic!("{name}: browser_timing {key} is not an integer: {row}"))
        };
        let text = |key: &str| {
            row[key]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: browser_timing {key} is not text: {row}"))
                .to_owned()
        };
        let timing = BrowserTiming {
            median_microseconds: number("median_microseconds"),
            maximum_microseconds: number("maximum_microseconds"),
            repetitions: number("repetitions"),
            engine: text("engine"),
            engine_version: text("engine_version"),
        };
        assert!(
            timing.repetitions > 0 && timing.median_microseconds <= timing.maximum_microseconds,
            "{name}: browser_timing for vector {index} is inconsistent: {row}"
        );
        timing
    }

    /// Require that every run the page and command-line comparison relies on
    /// passed, and that the SDK ran exactly the vectors the model declares.
    pub fn require(&self, vectors: usize, oversized: usize) {
        let document = &self.document;
        let name = self.path.display();
        for (what, value) in [
            ("status", &document["status"]),
            ("core_wasm.status", &document["core_wasm"]["status"]),
            ("hologram_oracle", &document["hologram_oracle"]),
            ("browser_projection", &document["browser_projection"]),
            ("artifact_closure", &document["artifact_closure"]),
            ("command_line.status", &document["command_line"]["status"]),
        ] {
            assert_eq!(value, "verified", "{name}: {what}");
        }
        let line = &document["command_line"];
        assert_eq!(
            document["modeled_vectors"], vectors,
            "{name}: modeled_vectors"
        );
        assert_eq!(line["vectors"], vectors, "{name}: command_line.vectors");
        assert_eq!(
            line["oversized_vectors"], oversized,
            "{name}: command_line.oversized_vectors"
        );
        assert_eq!(
            line["within_request_maximum"],
            vectors - oversized,
            "{name}: command_line.within_request_maximum"
        );
    }
}
