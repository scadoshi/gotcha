mod camera;
mod domain;
mod gotcha;
mod run;

use std::path::Path;

#[cfg(target_os = "linux")]
use run::linux::run;
#[cfg(target_os = "macos")]
use run::macos::run;

/// Photos land here, next to where the program was started.
const OUTPUT_DIR: &str = "gotchas";

fn main() -> std::process::ExitCode {
    match run(Path::new(OUTPUT_DIR)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            std::process::ExitCode::FAILURE
        }
    }
}
