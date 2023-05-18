use uma_details_utility::image;
use uma_details_utility::image::detail::{HeaderTrimMode, ImageConfig};
use uma_details_utility::image::ImageMatrix;

fn main() -> anyhow::Result<()> {
    const DIR_PATH: &str = "examples/test_images";
    const OUT_DIR_PATH: &str = "examples/test_images/result";
    
    let start_instant = std::time::Instant::now();
    
    let mut detail =
        image::detail::HorseGirlFullDetailImage::from_path(DIR_PATH, 10, ImageConfig::default())?;

    println!(
        "Full details saved in {}.",
        detail.write_to_file(OUT_DIR_PATH, "result.png")?
    );

    detail.set_config(ImageConfig {
        header_trim_mode: None,
        do_merge_close_button: false,
        scaling_threshold_pixels: None,
    });

    println!(
        "Close button trimmed details saved in {}.",
        detail.write_to_file(OUT_DIR_PATH, "result_without_close_button.png")?
    );

    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimMarginOnly),
        do_merge_close_button: true,
        scaling_threshold_pixels: None,
    });
    
    println!(
        "Margin trimmed details saved in {}.",
        detail.write_to_file(OUT_DIR_PATH, "result_margin_trimmed.png")?
    );

    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimTitleBar),
        do_merge_close_button: true,
        scaling_threshold_pixels: None,
    });

    println!(
        "Title bar trimmed details saved in {}.",
        detail.write_to_file(OUT_DIR_PATH, "result_title_trimmed.png")?
    );
    
    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimMarginOnly),
        do_merge_close_button: false,
        scaling_threshold_pixels: None,
    });
    
    println!(
        "Margin and close button trimmed details saved in {}.",
        detail.write_to_file(OUT_DIR_PATH, "result_margin_trimmed_without_close_button.png")?
    );
    
    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimTitleBar),
        do_merge_close_button: false,
        scaling_threshold_pixels: None,
    });
    
    println!(
        "Title bar and close button trimmed details saved in {}.",
        detail.write_to_file(OUT_DIR_PATH, "result_title_trimmed_without_close_button.png")?
    );
    
    let elapsed_time = start_instant.elapsed();
    println!("Completed in {}ms", elapsed_time.as_millis());
    
    Ok(())
}
