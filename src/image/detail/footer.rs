use opencv::core::{Mat, MatTraitConst};

use crate::image::ImageMatrix;
use crate::image::{Result, SizeIdentifiableImage};

#[derive(Debug)]
pub struct FooterImage {
    pub(in crate::image) image_mat: Mat,
}

impl ImageMatrix for FooterImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        Ok(self.image_mat.clone())
    }
}

impl SizeIdentifiableImage for FooterImage {
    fn width(&self) -> i32 {
        self.image_mat.cols()
    }

    fn height(&self) -> i32 {
        self.image_mat.rows()
    }
}
