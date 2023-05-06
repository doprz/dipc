use std::path::Path;

use image::AnimationDecoder;
use image::ImageDecoder;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::{prelude::ParallelIterator, slice::ParallelSliceMut};

use crate::delta::{CLIDEMethod, Lab};

pub fn convert_default(
    input_path: &Path,
    output_path: &Path,
    image_format: image::ImageFormat,
    palettes_lab: &[Lab],
    deltae_method: CLIDEMethod,
) {
    const CHUNK: usize = 4;

    // Open image
    let mut image = match image::open(input_path) {
        Ok(i) => i.into_rgba8(),
        Err(err) => {
            eprintln!(
                "Encountered error while opening image at path {}: {}",
                input_path.display(),
                err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
            );
            std::process::exit(127)
        }
    };

    // LAB conversion moved into palette
    // Apply palettes to image
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
                .to_nearest_palette(palettes_lab, deltae::DEMethod::from(deltae_method))
                .to_rgb();
            bytes[..3].copy_from_slice(&new_rgb);
        });
    progress_bar_clone.finish();

    match image.save_with_format(output_path, image_format) {
        Ok(_) => println!("Saved image: {:?}", output_path.display()),
        Err(err) => {
            eprintln!(
                "Encountered error while trying to save image \"{}\": {}",
                output_path.display(),
                err.if_supports_color(owo_colors::Stream::Stderr, |text| text.red())
            );
            std::process::exit(127)
        }
    };
}

fn gif_frame_count(input_path: &Path) -> usize {
    let file = std::fs::File::open(input_path).expect("Error opening file");
    let decoder = image::codecs::gif::GifDecoder::new(&file).expect("Error decoding GIF");
    decoder.into_frames().count()
}

pub fn convert_gif(
    input_path: &Path,
    output_path: &Path,
    palettes_lab: &[Lab],
    deltae_method: CLIDEMethod,
) {
    const CHUNK: usize = 4;
    let file = std::fs::File::open(input_path).expect("Error opening file");
    let decoder = image::codecs::gif::GifDecoder::new(&file).expect("Error decoding GIF");

    println!("Dimensions {:?}", decoder.dimensions());

    let frames = decoder.into_frames();
    let frame_count = gif_frame_count(input_path);
    let mut output_frames = Vec::new();

    let progress_bar = ProgressBar::new(frame_count as u64);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta_precise})",
        )
        .expect("Failed to set progress bar style"),
    );

    for (_idx, frame) in frames.enumerate() {
        match frame {
            Ok(frame) => {
                let mut image = frame.into_buffer();
                image.par_chunks_mut(CHUNK).for_each(|bytes| {
                    let pixel: [u8; CHUNK] =
                        bytes.try_into().expect("Error converting bytes to pixel");
                    let lab = Lab::from(pixel);
                    let new_rgb = lab
                        .to_nearest_palette(palettes_lab, deltae::DEMethod::from(deltae_method))
                        .to_rgb();
                    bytes[..3].copy_from_slice(&new_rgb);
                });

                output_frames.push(image::Frame::new(image));
                progress_bar.inc(1);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    progress_bar.finish();

    let gif_out = std::fs::File::create(output_path).expect("Error creating output file");
    let mut encoder = image::codecs::gif::GifEncoder::new(gif_out);
    encoder
        .set_repeat(image::codecs::gif::Repeat::Infinite)
        .expect("Error setting gif encoder repeat");
    encoder.encode_frames(output_frames).unwrap();
}
