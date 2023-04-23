use std::{
    ffi::OsStr,
    io::{self, stdout, BufWriter, Write},
};

use clap::Parser;
use cli::ColorPalette;
use delta::Lab;
use owo_colors::{OwoColorize, Style};
use rayon::{
    prelude::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
    },
    slice::{ParallelSlice, ParallelSliceMut},
};

use crate::{cli::Cli, config::parse_palette};

mod cli;
mod config;
mod delta;
mod palettes;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let Cli {
        color_palette,
        styles,
        mut output,
        process,
        verbose,
    } = cli;
    if process.is_empty() {
        eprintln!(
            "{}",
            "You need to provide at least a single image to process"
                .if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
        );
        std::process::exit(127)
    };
    if !output.is_dir() {
        eprintln!(
            "Provided output `{}` does not appear to be a directory.\nAttempting to create it!",
            output
                .display()
                .if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
        );
        match std::fs::create_dir_all(&output) {
            Ok(()) => {}
            Err(err) => {
                eprintln!(
                    "Creating provided output directory failed with error: {}",
                    err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                );
                std::process::exit(127)
            }
        };
    }
    let stdout = stdout().lock();
    let mut writer = BufWriter::new(stdout);
    if verbose >= 2 {
        writeln!(
            writer,
            "\
Using color palette: {color_palette:?}
With styles: {styles:?}"
        )?;
    }
    if verbose >= 3 {
        writeln!(
            writer,
            "\
To process {process:?}
And writing results to {output:?}"
        )?;
    };
    let mut name = {
        if !matches!(color_palette, ColorPalette::RawJSON { .. }) {
            format!("{color_palette:?}")
        } else {
            String::new()
        }
    };
    let palettes = match parse_palette(color_palette.get_json(), styles) {
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
    for palette in &palettes {
        if let Some(name) = &palette.name {
            writeln!(
                writer,
                "{:<10} - {} colors",
                name.if_supports_color(owo_colors::Stream::Stdout, |text| {
                    let style = Style::new().bold().bright_white();
                    text.style(style)
                }),
                palette.colors.len()
            )?;
        }
        const WIDTH: usize = 60;
        let mut x = 0;
        match supports_color::on_cached(supports_color::Stream::Stdout) {
            None => {}
            Some(level) => {
                if level.has_16m {
                    for (name, color) in &palette.colors {
                        let [r, g, b] = color.0;
                        let sum = r as u16 + g as u16 + b as u16;
                        let (fgr, fgg, fgb) = if sum > 128 * 3 {
                            (0, 0, 0)
                        } else {
                            (255, 255, 255)
                        };
                        x += name.len() + 1;
                        write!(
                            writer,
                            "{}{} ",
                            {
                                if x > WIDTH {
                                    x = 0;
                                    "\n"
                                } else {
                                    ""
                                }
                            },
                            name.on_truecolor(color.0[0], color.0[1], color.0[2])
                                .truecolor(fgr, fgg, fgb)
                        )?;
                    }
                }
            }
        }
        if x != 0 {
            writeln!(writer)?;
        }
    }
    writer.flush()?;
    palettes.iter().for_each(|p| {
        p.name.as_ref().map(|n| {
            name.push('-');
            name.push_str(n)
        });
    });
    let palettes: Vec<_> = palettes
        .into_par_iter()
        .flat_map_iter(|palette| {
            palette
                .colors
                .into_iter()
                .map(|(_name, color)| Lab::from(color.0))
        })
        .collect();
    for path in process {
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
        let filename = path.file_stem().unwrap_or(&OsStr::new("image"));
        const CHUNK: usize = 4;
        // Convert image to LAB representation
        let mut lab = Vec::with_capacity(image.as_raw().len() / 4);
        image
            .par_chunks_exact(CHUNK)
            .map(|pixel| {
                let pixel: [u8; CHUNK] = pixel.try_into().unwrap();
                Lab::from(pixel)
            })
            .collect_into_vec(&mut lab);
        // Apply palettes to image
        // let (name, colors) = palette;
        lab.par_iter()
            .zip_eq(image.par_chunks_exact_mut(CHUNK))
            .for_each(|(&lab, bytes)| {
                let new_rgb = lab.to_nearest_palette(&palettes).to_rgb();
                bytes[..3].copy_from_slice(&new_rgb);
            });

        let mut new_name = filename.to_os_string();
        new_name.push("-");
        new_name.push(&name);
        new_name.push(".png");
        output.push(new_name);
        match image.save_with_format(&output, image::ImageFormat::Png) {
            Ok(_) => {}
            Err(err) => {
                eprintln!(
                    "Encountered error while trying to save image `{}`: {}",
                    path.display()
                        .if_supports_color(owo_colors::Stream::Stderr, |text| text.blue()),
                    err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
                );
                std::process::exit(127)
            }
        };
        output.pop();
    }
    Ok(())
}
