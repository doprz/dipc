use std::io::{self, stdout, BufWriter, Read, Write};

use clap::Parser;
use delta::Lab;
use image::{codecs::png::PngEncoder, ImageEncoder};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use owo_colors::{OwoColorize, Style};
use rayon::{
    prelude::{IntoParallelRefIterator, ParallelIterator},
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

/// Check if path represents stdin/stdout
fn is_stdio(path: &std::path::Path) -> bool {
    path.as_os_str() == "-"
}

fn main() -> io::Result<()> {
    let total_start = std::time::Instant::now();
    let cli = Cli::parse();

    let stdout = stdout().lock();
    let mut writer = BufWriter::new(stdout);

    // Determine if we're outputting image to stdout
    let output_to_stdout = cli
        .output
        .as_ref()
        .is_some_and(|v| v.len() == 1 && is_stdio(&v[0]));

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

    if !output_to_stdout {
        println!(
            "Color palette: {}\nStyles: {:?}\nDeltaE method: {}",
            cli.color_palette, cli.styles, cli.method
        );
    }
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

    if !output_to_stdout {
        if let Some(path) = &cli.dir_output {
            println!("Writing results to {path:#?} directory.");
        }
        println!("Processing {:#?}", &cli.process);
        if let Some(output_vec) = &cli.output {
            println!("Output names: {output_vec:#?}");
        }
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

    // Print palettes (skip when piping to stdout)
    if !output_to_stdout {
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
    }

    // Remove duplicate colors
    for palette in &mut palettes {
        palette.colors.sort_by_key(|(_name, color)| color.0);
        palette.colors.dedup_by_key(|(_name, color)| color.0)
    }
    writer.flush()?;

    let palettes_lab: Vec<_> = palettes
        .par_iter()
        .flat_map_iter(|palette| {
            palette
                .colors
                .iter()
                .map(|(_name, color)| Lab::from(color.0))
        })
        .collect();

    for (idx, path) in cli.process.iter().enumerate() {
        let start = std::time::Instant::now();

        // Open image (from stdin or file)
        let mut image = if is_stdio(path) {
            let mut buf = Vec::new();
            io::stdin().lock().read_to_end(&mut buf)?;
            match image::load_from_memory(&buf) {
                Ok(i) => i.into_rgba8(),
                Err(err) => {
                    eprintln!(
                        "Encountered error while reading image from stdin: {}",
                        err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                    );
                    std::process::exit(127)
                }
            }
        } else {
            match image::open(path) {
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
            }
        };

        if !output_to_stdout {
            println!(
                "[{}/{}] Converting image... (this may take a while)",
                idx + 1,
                cli.process.len()
            );
        }

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

        // Apply palettes to image (skip progress bar when piping to stdout)
        if output_to_stdout {
            image.par_chunks_exact_mut(CHUNK).for_each(|bytes| {
                let pixel: [u8; CHUNK] = bytes.try_into().unwrap();
                let lab = Lab::from(pixel);
                let new_rgb = lab
                    .to_nearest_palette(&palettes_lab, deltae::DEMethod::from(cli.method))
                    .to_rgb();
                bytes[..3].copy_from_slice(&new_rgb);
            });
        } else {
            let progress_bar = ProgressBar::new(
                (image.len() / CHUNK)
                    .try_into()
                    .expect("Failed to convert usize to u64"),
            );
            progress_bar.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta_precise})",
                )
                .expect("Failed to set progress bar style"),
            );
            let progress_bar_clone = progress_bar.clone();
            image
                .par_chunks_exact_mut(CHUNK)
                .progress_with(progress_bar)
                .for_each(|bytes| {
                    let pixel: [u8; CHUNK] = bytes.try_into().unwrap();
                    let lab = Lab::from(pixel);
                    let new_rgb = lab
                        .to_nearest_palette(&palettes_lab, deltae::DEMethod::from(cli.method))
                        .to_rgb();
                    bytes[..3].copy_from_slice(&new_rgb);
                });
            progress_bar_clone.finish();
        }

        let output_file_name = match &cli.output {
            Some(output_vec) => {
                let mut name = output_vec[idx].clone();
                if !is_stdio(&name) {
                    name.set_extension("png");
                }
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
                    &cli.color_palette,
                    &palettes,
                    deltae::DEMethod::from(cli.method),
                ));
                output
            }
        };

        if is_stdio(&output_file_name) {
            let mut buf = Vec::new();
            let encoder = PngEncoder::new(&mut buf);
            if let Err(err) = encoder.write_image(
                &image,
                image.width(),
                image.height(),
                image::ColorType::Rgba8,
            ) {
                eprintln!(
                    "Encountered error while encoding image: {}",
                    err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                );
                std::process::exit(127)
            }
            io::stdout().lock().write_all(&buf)?;
        } else {
            match image.save_with_format(&output_file_name, image::ImageFormat::Png) {
                Ok(_) => {
                    if !output_to_stdout {
                        println!("Saved image: {:?}", output_file_name.display());
                    }
                }
                Err(err) => {
                    eprintln!(
                        "Encountered error while trying to save image \"{}\": {}",
                        output_file_name.display(),
                        err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                    );
                    std::process::exit(127)
                }
            };
        }

        if !output_to_stdout && cli.verbose >= 1 {
            let duration = start.elapsed().as_secs_f32();
            println!("Conversion took {duration} seconds.");
        }
    }

    if !output_to_stdout && cli.verbose >= 1 {
        let duration = total_start.elapsed().as_secs_f32();
        println!("Total duration: {duration} seconds.");
    }

    Ok(())
}
