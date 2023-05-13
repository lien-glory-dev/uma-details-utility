use uma_details_utility::image;
use uma_details_utility::image::detail::ImageConfig;
use uma_details_utility::image::ImageMatrix;

fn main() -> anyhow::Result<()> {
    const DIR_PATH: &str = "examples/merge_factors_screenshots";

    let detail =
        image::detail::HorseGirlFullDetailImage::from_path(DIR_PATH, 10, ImageConfig::default())?;
    
    println!(
        "Full details saved in {}.",
        detail.write_to_file(DIR_PATH, "result.png")?
    );

    let detail_close_button_trimmed =
        image::detail::HorseGirlFullDetailImage::from_path(DIR_PATH, 10, ImageConfig { do_trim_margin: false, do_merge_close_button: false })?;
    
    println!(
        "Close button trimmed details saved in {}.",
        detail_close_button_trimmed.write_to_file(DIR_PATH, "result_without_close_button.png")?
    );

    Ok(())
}
