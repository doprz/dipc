use std::io::{self, stdout, BufWriter, Write};

use clap::Parser;
use owo_colors::{OwoColorize, Style};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{
    cli::Cli,
    config::{output_file_name, parse_palette},
    convert_image_format::{convert_default, convert_gif},
    delta::Lab,
};

mod cli;
mod config;
mod convert_image_format;
mod delta;
mod palettes;

fn main() -> io::Result<()> {
    let total_start = std::time::Instant::now();
    let cli = Cli::parse();

    let stdout = stdout().lock();
    let mut writer = BufWriter::new(stdout);

    if cli.process.is_empty() {
        eprintln!(
            "{}",
            "You need to provide at least a single image to process"
                .if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
        );
        std::process::exit(127)
    };
    if let Some(output_vec) = &cli.output {
        if output_vec.is_empty() {
            eprintln!(
                "{}",
                "You need to provide at least a single output image name or path"
                    .if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
            );
        }
    }
    match &cli.output {
        Some(output_vec) if output_vec.len() != cli.process.len() => {
            eprintln!(
                "{}",
                "You need to provide the same amount of output image names/paths as input images"
                    .if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
            );
            std::process::exit(127)
        }
        _ => {}
    }

    println!(
        "Color palette: {}\nStyles: {:?}\nDeltaE method: {}",
        cli.color_palette, cli.styles, cli.method
    );
    match &cli.dir_output {
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
    if let Some(path) = &cli.dir_output {
        println!("Writing results to {:#?} directory.", path);
    }
    println!("Processing {:#?}", &cli.process);
    if let Some(output_vec) = &cli.output {
        println!("Output names: {:#?}", output_vec);
    }

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
        println!(
            "[{}/{}] Converting image... (this may take a while)",
            idx + 1,
            cli.process.len()
        );

        let path_extension: image::ImageFormat = match image::ImageFormat::from_path(path) {
            Ok(format) => format,
            Err(err) => {
                eprintln!(
                    "Failed to get extension of file \"{}\" with error: {}",
                    path.display(),
                    err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                );
                std::process::exit(127)
            }
        };

        let output_file_name = match &cli.output {
            Some(output_vec) => {
                let mut name = output_vec[idx].clone();
                name.set_extension(path_extension.extensions_str()[0]);
                match &cli.dir_output {
                    Some(path) => {
                        let mut output = path.clone();
                        output.push(name);
                        output
                    }
                    None => {
                        let mut output = std::path::PathBuf::new();
                        output.push(name);
                        output
                    }
                }
            }
            None => {
                let mut output = std::path::PathBuf::new();
                output.push(output_file_name(
                    &cli.dir_output,
                    path,
                    path_extension.extensions_str()[0],
                    &cli.color_palette,
                    &palettes,
                    deltae::DEMethod::from(cli.method),
                ));
                output
            }
        };

        match path_extension {
            image::ImageFormat::Gif => {
                convert_gif(path, &output_file_name, &palettes_lab, cli.method);
            }
            _ => {
                convert_default(
                    path,
                    &output_file_name,
                    path_extension,
                    &palettes_lab,
                    cli.method,
                );
            }
        }

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
