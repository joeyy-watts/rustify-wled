// use reqwest;
// use md5;

// use std::io;
// use std::fs::File;

// fn download_file(url: &str) -> Result<Bytes, reqwest::Error> {
//     /**
//      * Returns a file from cache under /tmp, with url as the cache key.
//      * If cache doesn't exist, downloads and caches it.
//      */
//     // cache key is MD5 of the url
//     let cache_key = md5::compute(url);

//     // cache file path is /tmp/{package_name}/{cache_key}
//     let cache_file_path = format!("/tmp/{}/{:x}", env!("CARGO_PACKAGE_NAME"), cache_key);

//     // check if cache exists
//     if let Ok(content) = get_file(&cache_file_path) {
//         return Ok(content);
//     } else {
//         // download file
//         let content = reqwest::blocking::get(url)?;

//         // cache file
//         cache_file(&cache_file_path, &content);

//         Ok(content)
//     }
// }



// fn cache_file(file_path: &str, content: &str) -> io::Result<()> {
//     let mut file = File::create(file_path)?;
//     io::copy(&mut content.as_bytes(), &mut file)?;
//     Ok(())
// }

// fn get_file(file_path: &str) -> io::Result<String> {
//     let file = File::open(file_path)?;
//     let reader = BufReader::new(file);
//     let mut content = String::new();

//     for line in reader.lines() {
//         content.push_str(&line?);
//         content.push('\n');
//     }

//     Ok(content)
// }