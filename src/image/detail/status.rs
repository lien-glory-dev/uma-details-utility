use opencv::core::Mat;

use crate::image::Result;
use crate::image::ImageMatrix;

#[derive(Debug)]
pub struct StatusImage {
    pub(in crate::image) image_mat: Mat,
}

impl ImageMatrix for StatusImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        Ok(self.image_mat.clone())
    }
}
