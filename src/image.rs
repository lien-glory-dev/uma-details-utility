use std::fmt::Debug;
use std::fs;

use opencv::core::MatTraitConst;
use thiserror::Error;

pub mod detail;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to load image from {path}")]
    LoadImageFromFileError { path: String, inner: opencv::Error },

    #[error("File {path} not found)")]
    FileNotFound { path: String },

    #[error("Not enough images")]
    NotEnoughImageSample,

    #[error("Required calculation is not completed: {message}")]
    RequiredCalculationsIsNotCompleted { message: String },

    #[error("Failed to matching images")]
    ImageNotMatched,

    #[error("Cv error: {source}")]
    CvError {
        #[from]
        source: opencv::Error,
    },

    #[error("File I/O error: {source}")]
    FileIoError {
        #[from]
        source: std::io::Error,
    },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
pub struct CropX(i32);

#[derive(Debug, Copy, Clone)]
pub struct CropWidth(i32);

#[derive(Debug, Copy, Clone)]
pub struct CropY(i32);

#[derive(Debug, Copy, Clone)]
pub struct CropHeight(i32);

pub trait ImageMatrix: Debug {
    fn convert_to_mat(&self) -> Result<opencv::core::Mat>;

    fn write_to_file(&self, dir_path: &str, name: &str) -> Result<String> {
        fs::create_dir_all(dir_path)?;

        let file_path = format!("{}/{}", dir_path, name);
        opencv::imgcodecs::imwrite(
            file_path.as_str(),
            &self.convert_to_mat()?,
            &opencv::core::Vector::new(),
        )?;

        Ok(file_path)
    }

    fn get_merged_below(&self, other: &dyn ImageMatrix) -> Result<SimpleImage> {
        let mut merged_image = opencv::core::Mat::default();
        opencv::core::vconcat2(
            &self.convert_to_mat()?,
            &other.convert_to_mat()?,
            &mut merged_image,
        )?;

        Ok(SimpleImage(merged_image))
    }
}

pub trait SizeIdentifiableImage: ImageMatrix {
    fn width(&self) -> i32;
    fn height(&self) -> i32;

    fn vertical_crop_image(
        &self,
        crop_y: CropY,
        crop_height: CropHeight,
    ) -> Result<opencv::core::Mat> {
        let cropping_rect = Rect::new(0, crop_y.0, self.width(), crop_height.0);
        let cropped_image = opencv::core::Mat::roi(&self.convert_to_mat()?, cropping_rect.into())?;

        Ok(cropped_image)
    }
    
    fn horizontal_crop_image(&self, crop_x: CropX, crop_width: CropWidth) -> Result<opencv::core::Mat> {
        let cropping_rect = Rect::new(crop_x.0, 0, crop_width.0, self.height());
        let cropped_image = opencv::core::Mat::roi(&self.convert_to_mat()?, cropping_rect.into())?;
        
        Ok(cropped_image)
    }
}

#[derive(Debug)]
pub struct SimpleImage(opencv::core::Mat);

impl SimpleImage {
    pub fn new(mat: opencv::core::Mat) -> Self {
        Self(mat)
    }
}

impl ImageMatrix for SimpleImage {
    fn convert_to_mat(&self) -> Result<opencv::core::Mat> {
        Ok(self.0.clone())
    }
}

impl SizeIdentifiableImage for SimpleImage {
    fn width(&self) -> i32 {
        self.0.cols()
    }

    fn height(&self) -> i32 {
        self.0.rows()
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<Rect> for opencv::core::Rect {
    fn from(value: Rect) -> opencv::core::Rect {
        opencv::core::Rect {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}
