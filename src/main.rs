//! Gotcha: locks the keyboard and mouse while you are away and photographs
//! whoever touches them, until the secret is typed.

mod camera;
mod domain;
mod gotcha;
mod run;

use std::path::Path;

#[cfg(target_os = "linux")]
use run::linux as platform;
#[cfg(target_os = "macos")]
use run::macos as platform;

/// Photos land here, next to where the program was started.
const OUTPUT_DIR: &str = "gotchas";

fn main() -> anyhow::Result<()> {
    platform::run(Path::new(OUTPUT_DIR))
}
