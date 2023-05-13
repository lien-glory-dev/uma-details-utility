use opencv::core::Mat;

use crate::image::Result;
use crate::image::ImageMatrix;

#[derive(Debug)]
pub struct FooterImage {
    pub(in crate::image) image_mat: Mat,
}

impl ImageMatrix for FooterImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        Ok(self.image_mat.clone())
    }
}
