use opencv::core::Mat;
use opencv::imgcodecs::{imread, IMREAD_COLOR};

use crate::image::ImageMatrix;
use crate::image::{ImageError, Result};

pub struct HorseGirlDetailsImage {
    inner: Mat,
}

impl HorseGirlDetailsImage {
    pub fn load(path: &str) -> Result<Self> {
        let inner = imread(path, IMREAD_COLOR).map_err(|e| ImageError::LoadFromFileError {
            path: path.to_string(),
            inner: e,
        })?;

        Ok(Self { inner })
    }
}

impl From<Mat> for HorseGirlDetailsImage {
    fn from(value: Mat) -> Self {
        Self { inner: value }
    }
}

impl ImageMatrix for HorseGirlDetailsImage {
    fn to_mat(&self) -> &Mat {
        &self.inner
    }
}
