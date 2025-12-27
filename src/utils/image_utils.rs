use std::io::Write;

/// write image to file
pub fn write_image_to_file(image: &[u8], file_path: &str) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(file_path)?;
    file.write_all(image)?;
    Ok(())
}

/// get image format
pub fn get_image_format(mime: &str) -> &str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        _ => "jpg",
    }
}
