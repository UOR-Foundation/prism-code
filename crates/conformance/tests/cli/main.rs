//! The generated command line (`CL-01` through `CL-18`).
//!
//! The program is generated from the application model by `prismpm build`;
//! nothing in this repository implements it (R1). These tests drive the built
//! program and hold it to what the model declares: the acceptance vectors, the
//! exit-code map, the help sections, the examples and the typed diagnostics of
//! every usage and input or output failure (R5). Run `prismpm build` first, or
//! set `PRISM_CODE_CLI` to the program.

mod check;
mod failures;
mod help;
mod model;
mod program;
mod source;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// What every test that drives the program needs.
pub struct Fixture {
    /// The generated program.
    pub program: PathBuf,
    /// The modeled command line.
    pub cli: model::Cli,
    /// The workbench guidance the help must carry.
    pub guidance: model::Guidance,
}

/// The repository root.
pub fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
}

/// The model and the program, loaded once.
pub fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let cli = model::load(root());
        let guidance = model::load_guidance(root());
        let program = program::locate(root(), &cli.cargo_name);
        Fixture {
            program,
            cli,
            guidance,
        }
    })
}
