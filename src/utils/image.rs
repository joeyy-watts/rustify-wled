use std::error::Error;
use std::path::Path;
use reqwest::blocking;
use image::{DynamicImage, GenericImageView, io::Reader as ImageReader};
use md5::Digest;
use std::fs;

pub fn get_image(url: &str) -> Result<image::DynamicImage, Box<dyn Error>> {
    let cache_path = get_cache_path(url);

    if Path::new(&cache_path).exists() {
        Ok(image::open(cache_path)?)
    } else {
        let mut response = reqwest::blocking::get(url)?;
        let content = response.bytes()?;
        let img = image::load_from_memory(&content)?;

        // TODO: add cache rotation
        fs::create_dir_all(&cache_path)?;
        img.save(cache_path)?;

        Ok(img)
    }
}

fn get_cache_path(url: &str) -> String {
    let cache_key = md5::compute(url);
    format!("/tmp/{}/{:x}", env!("CARGO_PKG_NAME"), cache_key)
}