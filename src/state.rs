use crate::camera::Camera;
use jiff::{Timestamp, Unit, civil::DateTime, tz::TimeZone};
use std::fs::create_dir_all;

pub struct GotchaState {
    camera: Camera,
    last_shot_at: DateTime,
}

const OUTPUT_DIR: &str = "gotchas";

impl GotchaState {
    pub fn new() -> anyhow::Result<Self> {
        create_dir_all(OUTPUT_DIR)?;
        println!("Output directory created");
        Ok(Self {
            camera: Camera::new()?,
            last_shot_at: Timestamp::now().to_zoned(TimeZone::UTC).datetime(),
        })
    }

    pub fn maybe_shoot_and_save(&mut self) -> anyhow::Result<()> {
        let now = Timestamp::now().to_zoned(TimeZone::UTC).datetime();
        let span = now - self.last_shot_at;
        if span.total(Unit::Second)? >= 1.0 {
            let image = self.camera.shoot()?;
            let file_name = format!("{}.png", now.strftime("%Y-%m-%d_%H-%M-%S%.3f"));
            println!("Gotcha!");
            image.save(format!("{}/{}", OUTPUT_DIR, file_name))?;
            println!("Image {} saved", file_name);
            self.last_shot_at = now;
        }
        Ok(())
    }
}
