use thiserror::Error;

pub mod horse_girl_details_image;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Failed to load image from {path}")]
    LoadFromFileError { path: String, inner: opencv::Error },
}

type Result<T> = std::result::Result<T, ImageError>;

pub trait ImageMatrix {
    fn to_mat(&self) -> &opencv::core::Mat;
}
