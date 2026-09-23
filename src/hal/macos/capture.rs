use crate::hal::driver::{DriverError, ImageBuffer};
use crate::hal::macos::app_manager::{find_window_for_app, get_frontmost_app_name};
use crate::utils::image_ops::encode_image_base64;
use core_graphics::display::*;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_graphics::image::CGImage;
use image::{ImageBuffer as GenericImageBuffer, Rgba};
use foreign_types::ForeignType;

pub fn capture_isolated_window(
    app_id: Option<&str>,
    window_id: Option<u64>,
    format_str: &str,
) -> Result<ImageBuffer, DriverError> {
    let target_app = if app_id.is_none() && window_id.is_none() {
        Some(get_frontmost_app_name()?)
    } else {
        app_id.map(|s| s.to_string())
    };

    let win_info = find_window_for_app(target_app.as_deref(), window_id)?;

    unsafe {
        let cg_wid = win_info.window_id as CGWindowID;
        let capture_rect = CGRect::new(
            &CGPoint::new(win_info.bounds.x, win_info.bounds.y),
            &CGSize::new(win_info.bounds.width, win_info.bounds.height),
        );
        let mut image_ref = CGWindowListCreateImage(
            capture_rect,
            kCGWindowListOptionIncludingWindow,
            cg_wid,
            kCGWindowImageBoundsIgnoreFraming,
        );

        if image_ref.is_null() {
            image_ref = CGWindowListCreateImage(
                capture_rect,
                kCGWindowListOptionIncludingWindow,
                cg_wid,
                kCGWindowImageDefault,
            );
        }

        if image_ref.is_null() {
            return Err(DriverError::CaptureFailed(format!(
                "Failed to capture window image for '{}' (wid: {})",
                win_info.app_name, win_info.window_id
            )));
        }

        let cg_image = CGImage::from_ptr(image_ref);

        let width = cg_image.width() as u32;
        let height = cg_image.height() as u32;
        let bytes_per_row = cg_image.bytes_per_row();
        let bits_per_pixel = cg_image.bits_per_pixel();
        let data = cg_image.data();
        let bytes = data.bytes();

        let mut rgba_img: GenericImageBuffer<Rgba<u8>, Vec<u8>> = GenericImageBuffer::new(width, height);
        let bpp = (bits_per_pixel / 8) as usize;

        for y in 0..height {
            let row_start = (y as usize) * bytes_per_row;
            for x in 0..width {
                let pixel_start = row_start + (x as usize) * bpp;
                if pixel_start + 3 < bytes.len() {
                    let b = bytes[pixel_start];
                    let g = bytes[pixel_start + 1];
                    let r = bytes[pixel_start + 2];
                    let a = bytes[pixel_start + 3];
                    rgba_img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }

        let scale_factor = if win_info.bounds.width > 0.0 {
            (width as f64) / win_info.bounds.width
        } else {
            1.0
        };

        let dynamic_img = image::DynamicImage::ImageRgba8(rgba_img);
        let base64_data = encode_image_base64(&dynamic_img, format_str)
            .map_err(|e| DriverError::CaptureFailed(e))?;

        Ok(ImageBuffer {
            base64_data,
            width,
            height,
            scale_factor,
            format: format_str.to_string(),
            window_title: win_info.title,
            app_id: win_info.app_name,
        })
    }
}
