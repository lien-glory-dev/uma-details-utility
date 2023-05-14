use opencv::core::{
    absdiff, in_range, Mat, MatTraitConst, MatTraitConstManual, Point, Rect as cvRect, Scalar,
};
use opencv::imgcodecs::{imread, IMREAD_COLOR};
use opencv::imgproc;
use opencv::types::VectorOfVectorOfPoint;

use factor::FactorListPartialImage;
use footer::FooterImage;
use status::StatusImage;

use crate::image::detail::factor::FactorListImage;
use crate::image::{CropHeight, CropWidth, CropX, CropY, Error, Result, SizeIdentifiableImage};
use crate::image::{ImageMatrix, Rect};

pub mod factor;
pub mod footer;
pub mod status;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HeaderTrimMode {
    TrimMarginOnly,
    TrimTitleBar,
}

#[derive(Debug, Copy, Clone)]
pub struct ImageConfig {
    pub header_trim_mode: Option<HeaderTrimMode>,
    pub do_merge_close_button: bool,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            header_trim_mode: Default::default(),
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
    const BINARY_BRIGHTNESS_THRESHOLD: u8 = 127;
    const BINARY_BLACK: u8 = 0;
    const BINARY_WHITE: u8 = 255;

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
        let factor_list_area = self.get_factor_list_area()?;

        let mut hsv_image = Mat::default();
        imgproc::cvt_color(
            &self.image_mat,
            &mut hsv_image,
            imgproc::COLOR_BGR2HSV,
            self.image_mat.channels(),
        )?;

        let mut binary_image = Mat::default();
        in_range(
            &hsv_image,
            &Scalar::all(0.0),
            &Scalar::new(250.0, 140.0, 240.0, 255.0),
            &mut binary_image,
        )?;

        let binary_image = Mat::roi(
            &binary_image,
            cvRect::new(
                0,
                factor_list_area.y,
                factor_list_area.x,
                factor_list_area.height,
            ),
        )?;

        let margin_end_points: Vec<usize> = (0..binary_image.rows())
            .filter_map(|y| {
                let rows = binary_image.row(y).ok()?;
                let rows = rows.data_bytes().ok()?;

                Self::get_position_black_after_white(rows)
            })
            .collect();

        if margin_end_points.is_empty() {
            return Err(Error::ImageNotMatched);
        }

        let margin_end_point = margin_end_points.iter().sum::<usize>() / margin_end_points.len();

        Ok(margin_end_point as i32)
    }

    pub fn get_top_margin(&self, include_title_bar: bool) -> Result<i32> {
        let factor_list_area = self.get_factor_list_area()?;

        let mut hsv_image = Mat::default();
        imgproc::cvt_color(
            &self.image_mat,
            &mut hsv_image,
            imgproc::COLOR_BGR2HSV,
            self.image_mat.channels(),
        )?;

        let mut binary_image = Mat::default();
        in_range(
            &hsv_image,
            &Scalar::new(30.0, 210.0, 160.0, 255.0),
            &Scalar::new(58.0, 255.0, 255.0, 255.0),
            &mut binary_image,
        )?;

        let binary_image = Mat::roi(
            &binary_image,
            cvRect::new(
                factor_list_area.x,
                0,
                factor_list_area.width,
                binary_image.rows() / 2,
            ),
        )?;

        let mut margin_end_points: Vec<usize> = (0..binary_image.cols())
            .filter_map(|x| {
                let cols = binary_image.col(x).ok()?;
                let cols: Vec<u8> = cols.iter::<u8>().ok()?.map(|(_, bin)| bin).collect();
                let cols = cols.as_slice();

                if include_title_bar {
                    Self::get_position_black_after_white(cols)
                } else {
                    Self::get_position_white_after_black(cols)
                }
            })
            .collect();

        if margin_end_points.is_empty() {
            return Err(Error::ImageNotMatched);
        }

        margin_end_points.sort();
        let margin_end_points = margin_end_points.split_at(margin_end_points.len() / 2).1;

        let margin_end_point =
            (margin_end_points.iter().sum::<usize>() / margin_end_points.len()) as i32;

        Ok(margin_end_point)
    }

    pub fn get_bottom_margin(&self) -> Result<i32> {
        let factor_list_area = self.get_factor_list_area()?;

        let mut hsv_image = Mat::default();
        imgproc::cvt_color(
            &self.image_mat,
            &mut hsv_image,
            imgproc::COLOR_BGR2HSV,
            self.image_mat.channels(),
        )?;

        let mut binary_image = Mat::default();
        in_range(
            &hsv_image,
            &Scalar::new(0.0, 0.0, 249.0, 255.0),
            &Scalar::new(0.0, 0.0, 249.5, 255.0),
            &mut binary_image,
        )?;

        let factor_list_area_end_y = factor_list_area.y + factor_list_area.height;
        let binary_image = Mat::roi(
            &binary_image,
            cvRect::new(
                factor_list_area.x,
                factor_list_area_end_y,
                factor_list_area.width,
                binary_image.rows() - factor_list_area_end_y,
            ),
        )?;

        let mut margin_end_points: Vec<usize> = (0..binary_image.cols())
            .filter_map(|x| {
                let cols = binary_image.col(x).ok()?;
                let mut cols: Vec<u8> = cols.iter::<u8>().ok()?.map(|(_, bin)| bin).collect();
                cols.reverse();
                let cols = cols.as_slice();

                Self::get_position_white_after_black(cols)
            })
            .collect();

        if margin_end_points.is_empty() {
            return Err(Error::ImageNotMatched);
        }

        margin_end_points.sort();
        let margin_end_points = margin_end_points.split_at(margin_end_points.len() / 2).1;

        let margin_end_point =
            (margin_end_points.iter().sum::<usize>() / margin_end_points.len()) as i32;

        Ok(margin_end_point)
    }

    fn get_position_white_after_black(pixels: &[u8]) -> Option<usize> {
        let mut is_reached_black = false;
        pixels.iter().position(|pixel_brightness| {
            if !is_reached_black && *pixel_brightness < Self::BINARY_BRIGHTNESS_THRESHOLD {
                is_reached_black = true;
            }

            is_reached_black && *pixel_brightness >= Self::BINARY_BRIGHTNESS_THRESHOLD
        })
    }

    fn get_position_black_after_white(pixels: &[u8]) -> Option<usize> {
        let mut is_reached_white = false;
        let result = pixels.iter().position(|pixel_brightness| {
            if !is_reached_white && *pixel_brightness > Self::BINARY_BRIGHTNESS_THRESHOLD {
                is_reached_white = true;
            }

            is_reached_white && *pixel_brightness <= Self::BINARY_BRIGHTNESS_THRESHOLD
        });

        result
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

    pub fn set_config(&mut self, config: ImageConfig) {
        self.config = config;
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

    pub fn get_top_margin(&self, include_title_bar: bool) -> Result<i32> {
        self.images[0].get_top_margin(include_title_bar)
    }

    pub fn get_bottom_margin(&self) -> Result<i32> {
        self.images[0].get_bottom_margin()
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

        if let Some(trim_mode) = self.config.header_trim_mode {
            let margin_left_right = self.get_left_right_margin()?;
            let margin_top = self.get_top_margin(trim_mode == HeaderTrimMode::TrimTitleBar)?;
            let margin_bottom = self.get_bottom_margin()?;
            let crop_width = CropWidth(merged_image.width() - margin_left_right * 2);
            let crop_height = if self.config.do_merge_close_button {
                CropHeight(merged_image.height() - (margin_top + margin_bottom))
            } else {
                CropHeight(merged_image.height() - margin_top)
            };

            merged_image =
                merged_image.horizontal_crop_image(CropX(margin_left_right), crop_width)?;
            merged_image = merged_image.vertical_crop_image(CropY(margin_top), crop_height)?;
        }

        merged_image.convert_to_mat()
    }
}
