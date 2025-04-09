use rfd::AsyncFileDialog;

#[derive(Debug, Clone)]
pub enum LoadError {
    Cancelled,
    FileError,
}

pub async fn open_image() -> Result<image::DynamicImage, LoadError> {
    let file = AsyncFileDialog::new()
        .add_filter("Image Files", &["png", "jpg", "jpeg"])
        .pick_file()
        .await
        .ok_or(LoadError::Cancelled)?;

    let image = image::open(file.path()).or(Err(LoadError::FileError))?;
    Ok(image.into())
}
