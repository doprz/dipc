use std::io::{self, stdout, BufWriter, Write};

use clap::Parser;
use delta::Lab;
use owo_colors::{OwoColorize, Style};
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{
    cli::Cli,
    config::{output_file_name, parse_palette},
};

mod cli;
mod config;
mod delta;
mod palettes;

fn main() -> io::Result<()> {
    let total_start = std::time::Instant::now();
    let cli = Cli::parse();
    if cli.process.is_empty() {
        eprintln!(
            "{}",
            "You need to provide at least a single image to process"
                .if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
        );
        std::process::exit(127)
    };
    let stdout = stdout().lock();
    let mut writer = BufWriter::new(stdout);

    println!(
        "Color palette: {}\nStyles: {:?}",
        cli.color_palette, cli.styles
    );
    match &cli.output {
        Some(path) if !path.is_dir() => {
            eprintln!(
                "Output directory \"{}\" does not exist.\nAttempting to create it.",
                path.display()
            );
            if let Err(err) = std::fs::create_dir_all(path) {
                eprintln!(
                    "Creating provided output directory failed with error: {}",
                    err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                );
                std::process::exit(127)
            };
        }
        _ => {}
    }
    if let Some(path) = &cli.output {
        println!("Writing results to {:#?} directory.", path);
    }
    println!("Processing {:#?}", &cli.process);

    let mut palettes = match parse_palette(cli.color_palette.clone().get_json(), &cli.styles) {
        Ok(p) => p,
        Err(err) => {
            eprintln!(
                "{}",
                err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
            );
            std::process::exit(127)
        }
    };
    // Print palettes
    let color = match supports_color::on_cached(supports_color::Stream::Stdout) {
        Some(level) => level.has_16m,
        None => false,
    };
    let max_name = palettes
        .iter()
        .map(|p| p.name.as_ref().map(|n| n.len()).unwrap_or_default())
        .max()
        .unwrap_or_default();
    for palette in &palettes {
        if let Some(name) = &palette.name {
            writeln!(
                writer,
                "{:<max_name$} - {} colors{}",
                name.if_supports_color(owo_colors::Stream::Stdout, |text| {
                    let style = Style::new().bold().bright_white();
                    text.style(style)
                }),
                palette.colors.len(),
                if color { ":" } else { "" }
            )?;
        }
        const WIDTH: usize = 8;
        let mut idx = 0;
        if color {
            for (_, color) in &palette.colors {
                let [r, g, b] = color.0;
                write!(writer, "{}", "  ".on_truecolor(r, g, b))?;
                if idx % WIDTH == WIDTH - 1 {
                    writeln!(writer)?;
                }
                idx += 1;
            }
            writeln!(writer)?;
        }
    }
    // Remove duplicate colors
    for palette in &mut palettes {
        palette.colors.sort_by_key(|(_name, color)| color.0);
        palette.colors.dedup_by_key(|(_name, color)| color.0)
    }
    writer.flush()?;

    let palettes_lab: Vec<_> = palettes
        .clone()
        .into_par_iter()
        .flat_map_iter(|palette| {
            palette
                .colors
                .into_iter()
                .map(|(_name, color)| Lab::from(color.0))
        })
        .collect();
    for (idx, path) in cli.process.iter().enumerate() {
        let start = std::time::Instant::now();
        // Open image
        let mut image = match image::open(&path) {
            Ok(i) => i.into_rgba8(),
            Err(err) => {
                eprintln!(
                    "Encountered error while opening image at path {}: {}",
                    path.display()
                        .if_supports_color(owo_colors::Stream::Stderr, |text| text.blue()),
                    err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                );
                std::process::exit(127)
            }
        };

        println!(
            "[{}/{}] Converting image... (this may take a while)",
            idx + 1,
            cli.process.len()
        );

        const CHUNK: usize = 4;
        // Convert image to LAB representation
        // let mut lab = Vec::with_capacity(image.as_raw().len() / CHUNK);
        // image
        //     .par_chunks_exact(CHUNK)
        //     .map(|pixel| {
        //         let pixel: [u8; CHUNK] = pixel.try_into().unwrap();
        //         Lab::from(pixel)
        //     })
        //     .collect_into_vec(&mut lab);
        //
        // LAB conversion moved into palette
        //
        // Apply palettes to image
        image.par_chunks_exact_mut(CHUNK).for_each(|bytes| {
            let pixel: [u8; CHUNK] = bytes.try_into().unwrap();
            let lab = Lab::from(pixel);
            let new_rgb = lab.to_nearest_palette(&palettes_lab).to_rgb();
            bytes[..3].copy_from_slice(&new_rgb);
        });

        let output_file_name = match &cli.output {
            Some(path) => {
                let mut output = path.clone();
                output.push(output_file_name(&path, &cli.color_palette, &palettes));
                output
            }
            None => {
                let mut output = std::path::PathBuf::new();
                output.push(output_file_name(&path, &cli.color_palette, &palettes));
                output
            }
        };

        match image.save_with_format(&output_file_name, image::ImageFormat::Png) {
            Ok(_) => println!("Saved image: {}", output_file_name.display()),
            Err(err) => {
                eprintln!(
                    "Encountered error while trying to save image {}: {}",
                    path.display()
                        .if_supports_color(owo_colors::Stream::Stderr, |text| text.blue()),
                    err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                );
                std::process::exit(127)
            }
        };

        if cli.verbose >= 1 {
            let duration = start.elapsed().as_secs_f32();
            println!("Conversion took {} seconds.", duration);
        }
    }

    if cli.verbose >= 1 {
        let duration = total_start.elapsed().as_secs_f32();
        println!("Total duration: {} seconds.", duration);
    }

    Ok(())
}
