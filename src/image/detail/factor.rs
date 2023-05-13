use opencv::core::{min_max_loc, vconcat2, Mat, MatTraitConst};
use opencv::imgproc;

use crate::image::detail::{HorseGirlDetailImage, HorseGirlFullDetailImage};
use crate::image::{
    CropHeight, CropY, Error, ImageMatrix, Rect, Result, SimpleImage, SizeIdentifiableImage,
};

const HEIGHT_PARTITION_NUM: i32 = 10;
const MATCHING_THRESHOLD: f64 = 0.95;

#[derive(Debug, Clone)]
pub struct FactorListPartialImage {
    image_mat: Mat,
    factor_list_area: Rect,
}

#[derive(Debug, Copy, Clone)]
struct MatchedPoint(i32, i32);

impl FactorListPartialImage {
    pub fn from_detail(src: &HorseGirlDetailImage) -> Result<Self> {
        let mut factor_list_area = src.get_factor_list_area()?;
        let crop_area = Rect {
            x: 0,
            y: factor_list_area.y,
            width: src.width(),
            height: factor_list_area.height,
        };

        let cropped_image = Mat::roi(&src.image_mat, crop_area.into())?;

        factor_list_area.y = 0;
        factor_list_area.height = cropped_image.rows();

        Ok(Self {
            image_mat: cropped_image,
            factor_list_area,
        })
    }

    pub fn merge_below(&mut self, other: &Self) -> Result<()> {
        self.image_mat = self.get_merged_below(other)?;
        self.factor_list_area.height = self.image_mat.rows();

        Ok(())
    }

    pub fn get_merged_below(&self, other: &Self) -> Result<Mat> {
        let MatchedPoint(self_image_matching, other_image_matching) =
            self.detect_match_area(other)?;

        let trimmed_self_image =
            self.vertical_crop_image(CropY(0), CropHeight(self_image_matching))?;
        let trimmed_other_image = other.vertical_crop_image(
            CropY(other_image_matching),
            CropHeight(other.image_mat.rows() - other_image_matching),
        )?;

        let mut merged_image = Mat::default();
        vconcat2(&trimmed_self_image, &trimmed_other_image, &mut merged_image)?;

        Ok(merged_image)
    }

    pub fn get_matching_roi(&self) -> Result<SimpleImage> {
        let roi = SimpleImage::new(Mat::roi(&self.image_mat, self.factor_list_area.into())?);

        Ok(roi)
    }
    
    fn detect_match_area(&self, other: &FactorListPartialImage) -> Result<MatchedPoint> {
        let self_matching_roi = self.get_matching_roi()?;
        let other_matching_roi = other.get_matching_roi()?;

        let partition_height = other_matching_roi.height() / HEIGHT_PARTITION_NUM;

        for partition_num in 0..HEIGHT_PARTITION_NUM {
            let other_scanning_pos = partition_num * partition_height;
            let other_scanning_roi = other_matching_roi
                .vertical_crop_image(CropY(other_scanning_pos), CropHeight(partition_height))?;

            for self_scanning_pos in (0..(self_matching_roi.height() - partition_height)).rev() {
                let self_scanning_roi = self_matching_roi
                    .vertical_crop_image(CropY(self_scanning_pos), CropHeight(partition_height))?;

                let mut match_result = Mat::default();
                imgproc::match_template(
                    &self_scanning_roi,
                    &other_scanning_roi,
                    &mut match_result,
                    imgproc::TM_CCOEFF_NORMED,
                    &Mat::default(),
                )?;

                let mut max_val = 0.0;

                min_max_loc(
                    &match_result,
                    None,
                    Some(&mut max_val),
                    None,
                    None,
                    &Mat::default(),
                )?;

                if max_val > MATCHING_THRESHOLD {
                    return Ok(MatchedPoint(self_scanning_pos, other_scanning_pos));
                }
            }
        }

        Err(Error::ImageNotMatched)
    }
}

impl ImageMatrix for FactorListPartialImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        Ok(self.image_mat.clone())
    }
}

impl SizeIdentifiableImage for FactorListPartialImage {
    fn width(&self) -> i32 {
        self.image_mat.cols()
    }

    fn height(&self) -> i32 {
        self.image_mat.rows()
    }
}

#[derive(Debug)]
pub struct FactorListImage {
    images: Vec<FactorListPartialImage>,
}

impl FactorListImage {
    pub fn from_detail(src: &HorseGirlFullDetailImage) -> Result<Self> {
        let images: Result<Vec<FactorListPartialImage>> = src
            .images
            .iter()
            .map(FactorListPartialImage::from_detail)
            .collect();

        Ok(Self { images: images? })
    }

    pub fn push(&mut self, image: FactorListPartialImage) {
        self.images.push(image)
    }
}

impl ImageMatrix for FactorListImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        self.images
            .iter()
            .skip(1)
            .fold(
                Ok(&mut self.images[0].clone()),
                |first_image, second_image| -> Result<&mut FactorListPartialImage> {
                    let first_image = first_image?;
                    first_image.merge_below(second_image)?;

                    Ok(first_image)
                },
            )?
            .convert_to_mat()
    }
}
