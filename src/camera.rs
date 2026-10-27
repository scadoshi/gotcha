use image::{ImageBuffer, Rgb};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};

pub(crate) struct Camera(nokhwa::Camera);

const WARMUP_FRAME_COUNT: usize = 30;

impl Camera {
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

    pub(crate) fn shoot(&mut self) -> anyhow::Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        let image = self.0.frame()?.decode_image::<RgbFormat>()?;
        println!("Frame shot");
        Ok(image)
    }
}
