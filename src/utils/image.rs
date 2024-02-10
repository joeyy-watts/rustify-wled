use std::error::Error;
use std::path::Path;
use reqwest::blocking;
use image::{DynamicImage, ImageFormat};
use md5::Digest;
use std::fs;

pub fn get_image_pixels(url: &str, width: &u32, height: &u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let img = get_image_sized(url, width, height)?;
    let pixels = img.to_rgb8().pixels().map(|p| vec![p[0], p[1], p[2]]).flatten().collect();
    Ok(pixels)
}

pub fn get_image_sized(url: &str, width: &u32, height: &u32) -> Result<DynamicImage, Box<dyn Error>> {
    let img = get_image_raw(url)?;
    let img = img.resize_exact(*height, *width, image::imageops::FilterType::Lanczos3);
    Ok(img)
}

pub fn get_image_raw(url: &str) -> Result<image::DynamicImage, Box<dyn Error>> {
    let cache_path = format!("{}.png", get_cache_path(url, true));

    if Path::new(&cache_path).exists() {
        let image = image::open(cache_path)?;
        Ok(image)
    } else {
        let response = reqwest::blocking::get(url)?;
        let content = response.bytes()?;
        let img = image::load_from_memory(&content)?;

        // TODO: add cache rotation
        fs::create_dir_all(get_cache_path(url, false))?;
        img.save_with_format(Path::new(&cache_path), ImageFormat::Png)?;

        Ok(img)
    }
}

fn get_cache_path(url: &str, with_file: bool) -> String {
    let cache_key = md5::compute(url);

    if with_file {
        format!("/tmp/{}/{:x}.png", env!("CARGO_PKG_NAME"), cache_key)
    } else {
        format!("/tmp/{}/{:x}", env!("CARGO_PKG_NAME"), cache_key)
    }
}