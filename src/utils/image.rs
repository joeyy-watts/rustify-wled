use std::error::Error;
use std::path::Path;
use image::{DynamicImage, ImageFormat};
use std::{fs, thread};

pub fn get_image_pixels(url: Option<String>, width: &u32, height: &u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let img = match url {
        Some(url) => get_image_sized(url.as_str(), width, height)?,
        None => DynamicImage::new_rgb8(*width, *height),
    };

    let pixels = img.to_rgb8().pixels().map(|p| vec![p[0], p[1], p[2]]).flatten().collect();
    Ok(pixels)
}

pub fn get_image_sized(url: &str, width: &u32, height: &u32) -> Result<DynamicImage, Box<dyn Error>> {
    let img = get_image_raw(url)?;
    let img = img.resize_exact(*height, *width, image::imageops::FilterType::Lanczos3);
    Ok(img)
}

pub fn get_image_raw(url: &str) -> Result<image::DynamicImage, Box<dyn Error>> {
    let cache_path = get_cache_path(url, true);

    if Path::new(&cache_path).exists() {
        let image = image::open(cache_path)?;
        Ok(image)
    } else {
        let response = reqwest::blocking::get(url)?;
        let content = response.bytes()?;
        let img = image::load_from_memory(&content)?;

        // TODO: add cache rotation
        let image_for_cache = img.clone();
        let new_cache_path = get_cache_path(url, false);
        thread::spawn(move || {
            fs::create_dir_all(new_cache_path).unwrap();
            image_for_cache.save_with_format(Path::new(&cache_path), ImageFormat::Png).unwrap();
        });


        Ok(img)
    }
}

fn get_cache_path(url: &str, with_file: bool) -> String {
    let cache_key = md5::compute(url);

    if with_file {
        format!("/tmp/{}/{:x}.png", env!("CARGO_PKG_NAME"), cache_key)
    } else {
        format!("/tmp/{}", env!("CARGO_PKG_NAME"))
    }
}