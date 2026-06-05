use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::imageops::FilterType as ResizeFilter;
use image::{ExtendedColorType, ImageEncoder, ImageFormat};

use crate::error::{CaptureError, CaptureResult};
use crate::types::CaptureImage;

const PREVIEW_MAX_WIDTH: u32 = 1600;
const PREVIEW_JPEG_QUALITY: u8 = 72;

pub fn encode_png_fast(image: &image::RgbaImage) -> CaptureResult<CaptureImage> {
    let mut buffer = Vec::new();
    let mut writer = Cursor::new(&mut buffer);
    let encoder = PngEncoder::new_with_quality(
        &mut writer,
        CompressionType::Fast,
        FilterType::NoFilter,
    );
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

    Ok(CaptureImage {
        width: image.width(),
        height: image.height(),
        png_bytes: buffer,
        rgba_bytes: image.as_raw().to_vec(),
    })
}

pub fn encode_png(image: &image::RgbaImage) -> CaptureResult<CaptureImage> {
    encode_png_fast(image)
}

pub fn downscale_for_preview(image: &image::RgbaImage) -> image::RgbaImage {
    if image.width() <= PREVIEW_MAX_WIDTH {
        return image.clone();
    }

    let ratio = PREVIEW_MAX_WIDTH as f32 / image.width() as f32;
    let target_height = (image.height() as f32 * ratio).round().max(1.0) as u32;
    image::imageops::resize(
        image,
        PREVIEW_MAX_WIDTH,
        target_height,
        ResizeFilter::Triangle,
    )
}

pub fn encode_jpeg_preview(image: &image::RgbaImage) -> CaptureResult<Vec<u8>> {
    let rgb = image::DynamicImage::ImageRgba8(image.clone()).into_rgb8();
    let mut buffer = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, PREVIEW_JPEG_QUALITY);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
    Ok(buffer)
}

pub fn encode_cropped_png(cropped: &image::RgbaImage) -> CaptureResult<CaptureImage> {
    let mut buffer = Vec::new();
    cropped
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

    Ok(CaptureImage {
        width: cropped.width(),
        height: cropped.height(),
        png_bytes: buffer,
        rgba_bytes: cropped.as_raw().to_vec(),
    })
}
