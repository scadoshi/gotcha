//! Shoots and saves a photo when input arrives, at most once a second.

use crate::{
    camera::Camera,
    domain::{Shutter, file_name},
};
use jiff::Zoned;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const INTERVAL: Duration = Duration::from_secs(1);

pub struct Gotcha {
    camera: Camera,
    shutter: Shutter,
    output_dir: PathBuf,
}

impl Gotcha {
    /// Creates `output_dir` and warms the camera up.
    pub fn new(output_dir: &Path) -> anyhow::Result<Self> {
        create_dir_all(output_dir)?;
        println!("Output directory {} ready", output_dir.display());
        Ok(Self {
            camera: Camera::new()?,
            shutter: Shutter::new(INTERVAL),
            output_dir: output_dir.to_path_buf(),
        })
    }

    /// Shoots and saves a photo, unless one was shot within the last second.
    pub fn on_input(&mut self) -> anyhow::Result<()> {
        if !self.shutter.fire(Instant::now()) {
            return Ok(());
        }
        let image = self.camera.shoot()?;
        let file_name = file_name(&Zoned::now());
        image.save(self.output_dir.join(&file_name))?;
        println!("Gotcha! {file_name} saved");
        Ok(())
    }
}
