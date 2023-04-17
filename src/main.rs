use clap::Parser;
use image::RgbaImage;
use rayon::prelude::*;
use std::path::PathBuf;

use dipc::ColorPalette;

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
    color_palette_variation: Option<Vec<String>>,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let cli = Cli::parse();
    // println!("{:#?}", cli);

    let color_palettes = dipc::init_color_palettes();
    let color_palette = match color_palettes.get(&cli.color_palette.to_string()) {
        Some(palette) => palette,
        None => {
            eprintln!(
                "Error: Color palette {} not found.",
                &cli.color_palette.to_string()
            );
            std::process::exit(1);
        }
    };
    match &cli.color_palette_variation {
        Some(variations) => {
            println!("{} - {:#?}", &cli.color_palette.to_string(), variations);
        }
        None => {
            println!("{}", &cli.color_palette.to_string());
        }
    }

    color_palette.iter().for_each(|(name, palette)| {
        println!("{} - {} colors:", name, palette.len());
        dipc::ansi_paint_palette(palette);
    });

    let image: RgbaImage = match image::open(&cli.image) {
        Ok(image) => image.to_rgba8(),
        Err(e) => {
            eprintln!("Error opening image: {}", e);
            std::process::exit(1);
        }
    };

    match cli.color_palette_variation {
        Some(variations) => {
            let palette_variations = dipc::get_color_palette_variations(color_palette, &variations);
            // println!("{:#?}", palette_variations);

            let output_file_name =
                match dipc::output_file_name(&cli.image, &cli.color_palette, &variations) {
                    Ok(output_file_name) => output_file_name,
                    Err(e) => {
                        eprintln!("Error getting output file name: {}", e);
                        std::process::exit(1);
                    }
                };

            println!("Output file name: {}", output_file_name);
            println!("Converting image... (this may take a while)");

            let start = std::time::Instant::now();

            let palette_lab = dipc::convert_palette_to_labs(&palette_variations);
            let labs_image: Vec<dipc::Lab> = dipc::rgba_pixels_to_labs(&image);
            let converted_image: Vec<u8> = labs_image
                .par_iter()
                .map(|lab| dipc::convert_lab_to_palette(lab, &palette_lab))
                .flatten()
                .collect();

            match image::save_buffer_with_format(
                &output_file_name,
                &converted_image,
                image.width(),
                image.height(),
                image::ColorType::Rgba8,
                image::ImageFormat::Png,
            ) {
                Ok(_) => println!("Image converted successfully."),
                Err(e) => eprintln!("Error converting image: {}", e),
            }

            let duration = start.elapsed().as_secs_f32();
            println!("Conversion took {} seconds.", duration);
        }
        None => {
            println!("No color palette variation(s) specified.");
        }
    }
}
