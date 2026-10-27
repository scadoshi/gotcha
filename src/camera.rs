//! The webcam: opens the first camera through `nokhwa` and shoots RGB frames.

use std::fmt;

use image::{ImageBuffer, Rgb};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};

/// The first camera, streaming at its highest resolution and warmed up, so the
/// first shot is not a dark frame.
pub(crate) struct Camera(nokhwa::Camera);

/// Prints nothing from the device.
impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Camera").finish_non_exhaustive()
    }
}

/// Frames read and thrown away after opening, while exposure settles.
const WARMUP_FRAME_COUNT: usize = 30;

impl Camera {
    /// Opens camera 0 and reads `WARMUP_FRAME_COUNT` frames.
    #[expect(clippy::print_stdout, reason = "the CLI's own progress lines")]
    pub(crate) fn new() -> anyhow::Result<Self> {
        let mut camera = nokhwa::Camera::new(
            CameraIndex::Index(0),
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
        )?;
        camera.open_stream()?;
        println!("Camera initialized");
        for _ in 0..WARMUP_FRAME_COUNT {
            camera.frame()?;
        }
        println!("Camera warmed up");
        Ok(Self(camera))
    }

    /// Reads one frame and decodes it to RGB.
    #[expect(clippy::print_stdout, reason = "the CLI's own progress lines")]
    pub(crate) fn shoot(&mut self) -> anyhow::Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        let image = self.0.frame()?.decode_image::<RgbFormat>()?;
        println!("Frame shot");
        Ok(image)
    }
}
