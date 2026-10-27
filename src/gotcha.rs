//! Shoots and saves a photo when input arrives, at most once a second.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use jiff::Zoned;

use crate::camera::Camera;
use crate::domain::{self, Shutter};

const INTERVAL: Duration = Duration::from_secs(1);

/// The camera, the one-shot-a-second rule, and where photos go.
#[derive(Debug)]
pub(crate) struct Gotcha {
    camera: Camera,
    shutter: Shutter,
    output_dir: PathBuf,
}

impl Gotcha {
    /// Creates `output_dir` and warms the camera up.
    #[expect(clippy::print_stdout, reason = "the CLI's own progress lines")]
    pub(crate) fn new(output_dir: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(output_dir)?;
        println!("Output directory {} ready", output_dir.display());
        Ok(Self {
            camera: Camera::new()?,
            shutter: Shutter::new(INTERVAL),
            output_dir: output_dir.to_path_buf(),
        })
    }

    /// Shoots and saves a photo, unless one was shot within the last second.
    #[expect(clippy::print_stdout, reason = "the CLI's own progress lines")]
    pub(crate) fn on_input(&mut self) -> anyhow::Result<()> {
        if !self.shutter.fire(Instant::now()) {
            return Ok(());
        }
        let image = self.camera.shoot()?;
        let file_name = domain::file_name(&Zoned::now());
        image.save(self.output_dir.join(&file_name))?;
        println!("Gotcha! {file_name} saved");
        Ok(())
    }
}
