use uma_details_utility::image;
use uma_details_utility::image::detail::{HeaderTrimMode, ImageConfig};
use uma_details_utility::image::ImageMatrix;

fn main() -> anyhow::Result<()> {
    const DIR_PATH: &str = "examples/merge_factors_screenshots";

    let mut detail =
        image::detail::HorseGirlFullDetailImage::from_path(DIR_PATH, 10, ImageConfig::default())?;

    println!(
        "Full details saved in {}.",
        detail.write_to_file(DIR_PATH, "result.png")?
    );

    detail.set_config(ImageConfig {
        header_trim_mode: None,
        do_merge_close_button: false,
    });

    println!(
        "Close button trimmed details saved in {}.",
        detail.write_to_file(DIR_PATH, "result_without_close_button.png")?
    );

    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimMarginOnly),
        do_merge_close_button: true,
    });
    
    println!(
        "Margin trimmed details saved in {}.",
        detail.write_to_file(DIR_PATH, "result_margin_trimmed.png")?
    );

    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimTitleBar),
        do_merge_close_button: true,
    });

    println!(
        "Title bar trimmed details saved in {}.",
        detail.write_to_file(DIR_PATH, "result_title_trimmed.png")?
    );
    
    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimMarginOnly),
        do_merge_close_button: false,
    });
    
    println!(
        "Margin and close button trimmed details saved in {}.",
        detail.write_to_file(DIR_PATH, "result_margin_trimmed_without_close_button.png")?
    );
    
    detail.set_config(ImageConfig {
        header_trim_mode: Some(HeaderTrimMode::TrimTitleBar),
        do_merge_close_button: false,
    });
    
    println!(
        "Title bar and close button trimmed details saved in {}.",
        detail.write_to_file(DIR_PATH, "result_title_trimmed_without_close_button.png")?
    );
    
    Ok(())
}
