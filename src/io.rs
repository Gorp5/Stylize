use anyhow::Result;
use image::{io::Reader as ImgReader, DynamicImage, GenericImageView, RgbaImage};

pub fn load_image(path: &str) -> Result<(Vec<u8>, u32, u32)> {
    let img = ImgReader::open(path)?.decode()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.into_raw(), w, h))
}

pub fn save_image(path: &str, data: &[u8], w: u32, h: u32) -> Result<()> {
    let img = RgbaImage::from_raw(w, h, data.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Invalid image data"))?;
    img.save(path)?;
    Ok(())
}