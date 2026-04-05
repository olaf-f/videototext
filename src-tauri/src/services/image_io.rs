use image::DynamicImage;

use crate::error::AppResult;

pub fn decode_image(bytes: &[u8]) -> AppResult<DynamicImage> {
    Ok(image::load_from_memory(bytes)?)
}
