use opencv::core::{absdiff, in_range, Mat, MatTraitConst, Point, Range, Rect as cvRect, Scalar, Vector};
use opencv::imgcodecs::{imread, IMREAD_COLOR};
use opencv::imgproc;
use opencv::types::{VectorOfi32, VectorOfVectorOfPoint};

use factor::FactorListPartialImage;
use footer::FooterImage;
use status::StatusImage;

use crate::image::detail::factor::FactorListImage;
use crate::image::{Error, Result, SimpleImage, SizeIdentifiableImage};
use crate::image::{ImageMatrix, Rect};

pub mod factor;
pub mod footer;
pub mod status;

#[derive(Debug, Copy, Clone)]
pub struct ImageConfig {
    pub do_trim_margin: bool,
    pub do_merge_close_button: bool,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            do_trim_margin: false,
            do_merge_close_button: true,
        }
    }
}

#[derive(Debug)]
pub struct HorseGirlDetailImage {
    image_mat: Mat,
    factor_list_area: Option<Rect>,
}

impl HorseGirlDetailImage {
    pub fn from_path(path: &str) -> Result<Self> {
        let inner = imread(path, IMREAD_COLOR)
            .map_err(|e| Error::LoadImageFromFileError {
                path: path.to_string(),
                inner: e,
            })
            .and_then(|i| {
                if i.empty() {
                    Err(Error::FileNotFound {
                        path: path.to_string(),
                    })
                } else {
                    Ok(i)
                }
            })?;

        Ok(Self {
            image_mat: inner,
            factor_list_area: Default::default(),
        })
    }

    pub fn get_factor_list_area(&self) -> Result<Rect> {
        self.factor_list_area
            .ok_or(Error::RequiredCalculationsIsNotCompleted {
                message: "Run first HorseGirlFullDetailImage::calc_children_list_area()"
                    .to_string(),
            })
    }
    
    pub fn get_left_right_margin(&self) -> Result<i32> {
        let mut grayscale_image = Mat::default();
        imgproc::cvt_color(
            &self.image_mat,
            &mut grayscale_image,
            imgproc::COLOR_BGR2HSV,
            self.image_mat.channels(),
        )?;
        
        let mut binary_image = Mat::default();
        in_range(
            &grayscale_image,
            &Scalar::all(0.0),
            &Scalar::new(250.0, 140.0, 240.0, 255.0),
            &mut binary_image,
        )?;
        
        SimpleImage::new(binary_image).write_to_file("debug-images", "diff_threshold.png")?;
        panic!();
    }

    pub fn get_status_image(&self) -> Result<StatusImage> {
        let factor_list_area = self.get_factor_list_area()?;
        let status_area = cvRect::new(0, 0, self.image_mat.cols(), factor_list_area.y);
        let cropped_image = Mat::roi(&self.image_mat, status_area)?;

        Ok(StatusImage {
            image_mat: cropped_image,
        })
    }

    pub fn get_factor_image(&self) -> Result<FactorListPartialImage> {
        FactorListPartialImage::from_detail(self)
    }

    pub fn get_footer_image(&self) -> Result<FooterImage> {
        let factor_list_area = self.get_factor_list_area()?;

        let footer_start_y = factor_list_area.y + factor_list_area.height;
        let footer_area = cvRect::new(
            0,
            footer_start_y,
            self.image_mat.cols(),
            self.image_mat.rows() - footer_start_y,
        );

        let cropped_image = Mat::roi(&self.image_mat, footer_area)?;

        Ok(FooterImage {
            image_mat: cropped_image,
        })
    }

    fn diff_binary_mat(&self, other: &Self) -> Result<Mat> {
        let self_image = &self.image_mat;
        let other_image = &other.image_mat;

        let mut diff_image = Mat::default();
        absdiff(self_image, other_image, &mut diff_image)?;

        let mut diff_grayscale_image = Mat::default();
        imgproc::cvt_color(
            &diff_image,
            &mut diff_grayscale_image,
            imgproc::COLOR_BGR2GRAY,
            diff_image.channels(),
        )?;

        let mut diff_threshold_image = Mat::default();
        imgproc::threshold(
            &diff_grayscale_image,
            &mut diff_threshold_image,
            70.0,
            255.0,
            imgproc::THRESH_BINARY,
        )?;

        Ok(diff_threshold_image)
    }
}

impl ImageMatrix for HorseGirlDetailImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        Ok(self.image_mat.clone())
    }
}

impl SizeIdentifiableImage for HorseGirlDetailImage {
    fn width(&self) -> i32 {
        self.image_mat.cols()
    }

    fn height(&self) -> i32 {
        self.image_mat.rows()
    }
}

#[derive(Debug)]
pub struct HorseGirlFullDetailImage {
    images: Vec<HorseGirlDetailImage>,
    config: ImageConfig,
}

impl HorseGirlFullDetailImage {
    pub fn from_path(base_dir_path: &str, images_limit: i32, config: ImageConfig) -> Result<Self> {
        assert!(images_limit > 0, "images_limit must greater then 0");

        let mut images = Vec::new();

        for i in 1..=images_limit {
            let image_path = format!("{}/{}.png", base_dir_path, i);
            let image = HorseGirlDetailImage::from_path(image_path.as_str());

            if let Err(Error::FileNotFound { .. }) = image {
                break;
            }

            images.push(image?);
        }

        if images.len() < 2 {
            return Err(Error::NotEnoughImageSample);
        }

        let mut new = Self { images, config };
        new.calc_children_list_area()?;

        Ok(new)
    }

    pub fn calc_children_list_area(&mut self) -> Result<Rect> {
        let list_area_rect = self.get_list_area_rect()?;

        for image in &mut self.images {
            image.factor_list_area = Some(list_area_rect);
        }

        Ok(list_area_rect)
    }
    
    pub fn get_left_right_margin(&self) -> Result<i32> {
        self.images[0].get_left_right_margin()
    }
    
    pub fn get_status_image(&self) -> Result<StatusImage> {
        self.images[0].get_status_image()
    }
    
    pub fn get_factor_list_image(&self) -> Result<FactorListImage> {
        FactorListImage::from_detail(self)
    }
    
    pub fn get_footer_image(&self) -> Result<FooterImage> {
        self.images[0].get_footer_image()
    }

    fn get_list_area_rect(&self) -> Result<Rect> {
        let first_image = &self.images[0];
        let second_image = &self.images[1];

        let diff_threshold_image = first_image.diff_binary_mat(second_image)?;

        let mut diff_contours = VectorOfVectorOfPoint::new();
        imgproc::find_contours(
            &diff_threshold_image,
            &mut diff_contours,
            imgproc::RETR_TREE,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;

        let mut all_diff_covered_rect = Rect::default();

        for contour in diff_contours.iter() {
            let rect = imgproc::bounding_rect(&contour)?;

            if all_diff_covered_rect.x == Default::default() {
                all_diff_covered_rect.x = rect.x;
            }
            if all_diff_covered_rect.y == Default::default() {
                all_diff_covered_rect.y = rect.y;
            }
            if all_diff_covered_rect.width == Default::default() {
                all_diff_covered_rect.width = rect.width;
            }
            if all_diff_covered_rect.height == Default::default() {
                all_diff_covered_rect.height = rect.height;
            }

            if rect.x < all_diff_covered_rect.x {
                all_diff_covered_rect.width += all_diff_covered_rect.x - rect.x;
                all_diff_covered_rect.x = rect.x;
            }
            if rect.y < all_diff_covered_rect.y {
                all_diff_covered_rect.height += all_diff_covered_rect.y - rect.y;
                all_diff_covered_rect.y = rect.y;
            }
            if (rect.x + rect.width) > (all_diff_covered_rect.x + all_diff_covered_rect.width) {
                all_diff_covered_rect.width = rect.x + rect.width - all_diff_covered_rect.x;
            }
            if (rect.y + rect.height) > (all_diff_covered_rect.y + all_diff_covered_rect.height) {
                all_diff_covered_rect.height = rect.y + rect.height - all_diff_covered_rect.y;
            }
        }

        Ok(all_diff_covered_rect)
    }
}

impl ImageMatrix for HorseGirlFullDetailImage {
    fn convert_to_mat(&self) -> Result<Mat> {
        let status_image = self.get_status_image()?;
        let factor_image = self.get_factor_list_image()?;

        let mut merged_image = status_image.get_merged_below(&factor_image)?;

        if self.config.do_merge_close_button {
            let footer_image = self.get_footer_image()?;
            merged_image = merged_image.get_merged_below(&footer_image)?;
        }

        merged_image.convert_to_mat()
    }
}
