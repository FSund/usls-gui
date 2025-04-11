use std::path::Path;

use rfd::AsyncFileDialog;

#[derive(Debug, Clone)]
pub enum LoadError {
    Cancelled,
    FileError,
}

pub fn load_image(path: &Path) -> Result<image::DynamicImage, LoadError> {
    let image = image::open(path).or(Err(LoadError::FileError))?;
    Ok(image.into())
}

pub async fn open_image() -> Result<image::DynamicImage, LoadError> {
    let file = AsyncFileDialog::new()
        .add_filter("Image Files", &["png", "jpg", "jpeg"])
        .pick_file()
        .await
        .ok_or(LoadError::Cancelled)?;

    load_image(file.path())
}
