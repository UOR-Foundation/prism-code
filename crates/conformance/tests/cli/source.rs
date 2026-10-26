//! No handwritten implementation of the command line (`CL-04`).
//!
//! R1 allows no Rust that dispatches, parses requests or formats reports; the
//! program is generated. The scan reads every Rust file under `crates` and
//! `xtask`, this one included, for the strings only such an implementation
//! needs. They are assembled from parts so this file does not contain them.

use std::path::{Path, PathBuf};

use crate::root;

fn markers() -> Vec<String> {
    vec![
        ["status", "accepted"].join(" "),
        ["status", "rejected"].join(" "),
        ["status", "error"].join(" "),
        ["dataflow-policy", "1"].join("/"),
        ["split", "exact"].join("_"),
        format!("fn {}(", "dispatch"),
    ]
}

fn scan(text: &str, markers: &[String]) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for marker in markers {
            if line.contains(marker.as_str()) {
                found.push((index + 1, marker.clone()));
            }
        }
    }
    found
}

fn sources(directory: &Path, found: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
            .path();
        if path.is_dir() {
            if path.file_name().is_none_or(|name| name != "target") {
                sources(&path, found);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// `CL-04`: no Rust file names what an implementation would need.
#[test]
fn no_rust_source_implements_the_command_line_cl_04() {
    let markers = markers();
    let mut files = Vec::new();
    sources(&root().join("crates"), &mut files);
    sources(&root().join("xtask"), &mut files);
    assert!(
        files
            .iter()
            .any(|file| file.ends_with("tests/cli/source.rs")),
        "the scan must include its own file"
    );
    let mut violations = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        for (line, marker) in scan(&text, &markers) {
            violations.push(format!("{}:{line}: {marker}", file.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "R1: Rust source implements the command line:\n{}",
        violations.join("\n")
    );

    // The scan can fail: a planted implementation is found, a clean file is not.
    let planted = format!("fn {}(request: &[u8]) {{}}", "dispatch");
    assert!(!scan(&planted, &markers).is_empty());
    assert!(scan("fn check(request: &[u8]) {}", &markers).is_empty());
}
