use crate::server::status::Status;
use base64::{engine::general_purpose, Engine};
use std::error::Error;
use std::fs;
use image::GenericImageView;

pub struct CachedStatus {
    pub status: Status,
    pub json: String
}

impl CachedStatus {
    pub fn new() -> Self {
        let status = Status::build(
            -1, "hello from necko-core :3".into(), 
            match Self::build_favicon("icon.png") {
                Ok(data) => Some(data), _ => None
            }, false, false
        );
        let json = serde_json::to_string(&status)
            .expect("Could not serialize cache status.");

        Self { status, json }
    }

    pub fn build_favicon(image_path: &str) -> Result<String, Box<dyn Error>> {
        if !image_path.to_lowercase().ends_with(".png") {
            return Err("The icon must have the .png extension".into())
        }

        let img = image::open(image_path)?;
        if img.dimensions() != (64, 64) {
            return Err("The icon must be an 64x64 image".into())
        }

        let image_data = fs::read(image_path)?;
        let base64_data = general_purpose::STANDARD.encode(image_data);
        Ok(format!("data:image/png;base64,{base64_data}"))
    }
}