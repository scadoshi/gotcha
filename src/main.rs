mod camera;
mod run;
mod state;

#[cfg(target_os = "linux")]
use run::linux::run;

#[cfg(target_os = "macos")]
use run::macos::run;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
    }
}
