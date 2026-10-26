//! Locating and running the generated program.
//!
//! The program is the artifact `prismpm build` publishes for the host
//! architecture, so these tests drive exactly what a user runs. They do not
//! skip when it is absent: a claim about a program that was never built is not
//! a claim, so a missing program is a failure that says how to build it.

use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

/// Environment variable that names the program, overriding the build search.
pub const PROGRAM_VARIABLE: &str = "PRISM_CODE_CLI";

fn host_target() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64-unknown-linux-musl",
        "aarch64" => "aarch64-unknown-linux-musl",
        other => panic!("the SDK builds the command line for x86_64 and aarch64 only, not {other}"),
    }
}

/// The generated program named `binary`: `PRISM_CODE_CLI` when set, otherwise
/// the newest `cli/bin/<host target>/<binary>` under `.prism/build/`.
pub fn locate(root: &Path, binary: &str) -> PathBuf {
    if let Some(path) = std::env::var_os(PROGRAM_VARIABLE) {
        let path = PathBuf::from(path);
        assert!(
            path.is_file(),
            "{PROGRAM_VARIABLE}={} is not a file",
            path.display()
        );
        return path;
    }
    let builds = root.join(".prism").join("build");
    let entries = std::fs::read_dir(&builds).unwrap_or_else(|error| {
        panic!(
            "no generated {binary}: {}: {error}. Run `prismpm build` first, or set \
             {PROGRAM_VARIABLE} to the program",
            builds.display()
        )
    });
    let mut found: Vec<(SystemTime, PathBuf)> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", builds.display()));
        let candidate = entry
            .path()
            .join("cli")
            .join("bin")
            .join(host_target())
            .join(binary);
        if candidate.is_file() {
            let modified = candidate
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or_else(|error| panic!("{}: {error}", candidate.display()));
            found.push((modified, candidate));
        }
    }
    found.sort();
    match found.pop() {
        Some((_, path)) => path,
        None => panic!(
            "no generated {binary} for {} under {}. Run `prismpm build` first, or set \
             {PROGRAM_VARIABLE} to the program",
            host_target(),
            builds.display()
        ),
    }
}

/// What one run of the program produced.
#[derive(Debug)]
pub struct Output {
    /// The exit status.
    pub code: i32,
    /// Standard output, when it was captured.
    pub stdout: Vec<u8>,
    /// Standard error.
    pub stderr: Vec<u8>,
}

impl Output {
    /// Standard error as text.
    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }

    /// Standard output as text.
    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

/// Run `program` with explicit standard streams. Standard error is captured.
pub fn run_with(program: &Path, args: &[OsString], stdin: Stdio, stdout: Stdio) -> Output {
    let output = Command::new(program)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(Stdio::piped())
        .output()
        .unwrap_or_else(|error| panic!("running {}: {error}", program.display()));
    let code = output
        .status
        .code()
        .unwrap_or_else(|| panic!("{} did not exit: {}", program.display(), output.status));
    Output {
        code,
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

/// A private directory for the files one test needs, removed when it drops.
pub struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    /// Create an empty directory under the system temporary directory.
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "prism-code-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
        Self { dir }
    }

    /// The directory itself.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Write `bytes` to a file called `name` and return its path.
    pub fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.dir.join(name);
        std::fs::write(&path, bytes).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        path
    }

    /// Standard input that reads `bytes`.
    pub fn stdin(&self, bytes: &[u8]) -> Stdio {
        let path = self.file("stdin", bytes);
        let file = File::open(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        Stdio::from(file)
    }
}

impl Default for Scratch {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Removal failure leaves a temporary directory behind and cannot change
        // what a test established, and a destructor has no way to report it.
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Arguments from string literals.
pub fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

/// Run `program` with `args`, standard input reading `stdin` and standard
/// output captured.
pub fn run(program: &Path, args: &[OsString], stdin: &[u8]) -> Output {
    let scratch = Scratch::new();
    run_with(program, args, scratch.stdin(stdin), Stdio::piped())
}
