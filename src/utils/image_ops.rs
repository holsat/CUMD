use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use crate::utils::coordinates::Rect;

pub fn crop_to_window(image: &DynamicImage, rect: &Rect, scale: f64) -> DynamicImage {
    let px = (rect.x * scale).max(0.0) as u32;
    let py = (rect.y * scale).max(0.0) as u32;
    let pwidth = (rect.width * scale) as u32;
    let pheight = (rect.height * scale) as u32;

    let (img_w, img_h) = image.dimensions();
    let crop_w = pwidth.min(img_w.saturating_sub(px));
    let crop_h = pheight.min(img_h.saturating_sub(py));

    if crop_w == 0 || crop_h == 0 {
        return image.clone();
    }

    image.crop_imm(px, py, crop_w, crop_h)
}

pub fn encode_image_base64(image: &DynamicImage, format_str: &str) -> Result<String, String> {
    let format = match format_str.to_lowercase().as_str() {
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        _ => ImageFormat::Png,
    };

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    image
        .write_to(&mut cursor, format)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(BASE64.encode(buffer))
}
