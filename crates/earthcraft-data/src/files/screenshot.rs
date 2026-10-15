//! Reserve screenshot names atomically on the file worker, without overwriting.
use super::FileOutcome;
use std::{
    fs::OpenOptions,
    io::{BufWriter, ErrorKind, Write},
    path::PathBuf,
};

pub(super) fn save(
    path: PathBuf,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<FileOutcome, crate::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut candidate = path.clone();
    let mut suffix = 0_u64;
    let file = loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => break file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                suffix += 1;
                let mut name = path
                    .file_stem()
                    .ok_or("Screenshot path has no filename")?
                    .to_os_string();
                name.push(format!("_{suffix}"));
                if let Some(extension) = path.extension() {
                    name.push(".");
                    name.push(extension);
                }
                candidate = path.with_file_name(name);
            }
            Err(error) => return Err(error.into()),
        }
    };
    let mut writer = BufWriter::new(file);
    image::write_buffer_with_format(
        &mut writer,
        pixels,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;
    writer.flush()?;
    Ok(FileOutcome::ScreenshotSaved { path: candidate })
}
