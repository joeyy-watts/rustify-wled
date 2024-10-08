use std::error::Error;
use std::path::Path;
use image::{DynamicImage, ImageFormat};
use std::{fs, thread};
use log::{debug, log, trace};

pub fn get_image_pixels(url: Option<String>, width: &u32, height: &u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let img = match url {
        Some(url) => get_image_sized(url.as_str(), width, height)?,
        None => DynamicImage::new_rgb8(*width, *height),
    };

    let pixels = img.to_rgb8().pixels().map(|p| vec![p[0], p[1], p[2]]).flatten().collect();
    Ok(pixels)
}

pub fn precache_image(url: &str) -> Result<(), Box<dyn Error>> {
    // check if cache already exists
    if cache_exists(url) {
        debug!("Cache already exists for {}", url);
        return Ok(());
    } else {
        debug!("Precaching image for {}", url);
        get_image_raw(url)?;
        Ok(())
    }
}

fn get_image_sized(url: &str, width: &u32, height: &u32) -> Result<DynamicImage, Box<dyn Error>> {
    let img = get_image_raw(url)?;
    let img = img.resize_exact(*height, *width, image::imageops::FilterType::Lanczos3);
    Ok(img)
}

fn get_image_raw(url: &str) -> Result<DynamicImage, Box<dyn Error>> {
    let cache_path = get_cache_path(url, true);

    if Path::new(&cache_path).exists() {
        trace!("Cache hit for {}", url);
        let image = image::open(cache_path)?;
        Ok(image)
    } else {
        trace!("Cache miss for {}, downloading", url);
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

fn cache_exists(url: &str) -> bool {
    let cache_path = get_cache_path(url, true);

    if Path::new(&cache_path).exists() {
        trace!("Cache exists for {}", url);
        true
    } else {
        trace!("Cache does not exist for {}", url);
        false
    }
}

fn get_cache_path(url: &str, with_file: bool) -> String {
    if with_file {
        let cache_key = md5::compute(url);
        format!("/tmp/{}/{:x}.png", env!("CARGO_PKG_NAME"), cache_key)
    } else {
        format!("/tmp/{}", env!("CARGO_PKG_NAME"))
    }
}