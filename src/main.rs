use clap::{Parser, ValueEnum};
use image::RgbaImage;
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash, Debug)]
enum ColorPalette {
    Catppuccin,
    Nord,
}

impl std::fmt::Display for ColorPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorPalette::Catppuccin => write!(f, "catppuccin"),
            ColorPalette::Nord => write!(f, "nord"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The image to process
    #[arg(short, long, value_name = "FILE")]
    image: PathBuf,

    /// The color palette to use
    #[arg(long, value_enum)]
    color_palette: ColorPalette,

    /// Color palette variation(s) to use
    #[arg(long)]
    color_palette_variation: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    // println!("{:#?}", cli);

    let color_palettes = dipc::init_color_palettes();
    let color_palette = color_palettes.get(&cli.color_palette.to_string()).unwrap();
    println!(
        "{} - {:#?}",
        &cli.color_palette.to_string(),
        &cli.color_palette_variation
    );

    color_palette.iter().for_each(|(name, palette)| {
        println!("{} - {} colors:", name, palette.len());
        dipc::ansi_paint_palette(palette);
    });

    let image: RgbaImage = match image::open(&cli.image) {
        Ok(image) => image.to_rgba8(),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let palette_variations =
        dipc::get_color_palette_variations(&color_palette, &cli.color_palette_variation);
    // println!("{:#?}", palette_variations);

    let palette_lab = dipc::convert_palette_to_labs(&palette_variations);

    let labs_image: Vec<dipc::Lab> = dipc::rgba_pixels_to_labs(&image);
    let converted_image: Vec<u8> = labs_image
        .iter()
        .map(|lab| dipc::convert_lab_to_palette(lab, &palette_lab))
        .flatten()
        .collect();
    image::save_buffer_with_format(
        "converted.png",
        &converted_image,
        image.width(),
        image.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();
}
