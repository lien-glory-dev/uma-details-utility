use opencv::core::{Mat, MatTraitConst};

use crate::image::ImageMatrix;
use crate::image::{Result, SizeIdentifiableImage};

#[derive(Debug)]
pub struct StatusImage {
    pub(in crate::image) image_mat: Mat,
}

impl ImageMatrix for StatusImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        Ok(self.image_mat.clone())
    }
}

impl SizeIdentifiableImage for StatusImage {
    fn width(&self) -> i32 {
        self.image_mat.cols()
    }

    fn height(&self) -> i32 {
        self.image_mat.rows()
    }
}
