use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash, Debug)]
pub enum ColorPalette {
    Catppuccin,
    Gruvbox,
    GruvboxMaterial,
    Nord,
}

impl std::fmt::Display for ColorPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorPalette::Catppuccin => write!(f, "catppuccin"),
            ColorPalette::Gruvbox => write!(f, "gruvbox"),
            ColorPalette::GruvboxMaterial => write!(f, "gruvbox-material"),
            ColorPalette::Nord => write!(f, "nord"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
    pub alpha: f32,
}

impl Lab {
    pub fn from_rgba(rgba: &[u8; 4]) -> Self {
        let lab = lab::Lab::from_rgba(rgba);
        Self {
            l: lab.l,
            a: lab.a,
            b: lab.b,
            alpha: rgba[3] as f32 / 255.0,
        }
    }

    pub fn to_rgba(&self) -> [u8; 4] {
        let lab = lab::Lab {
            l: self.l,
            a: self.a,
            b: self.b,
        };
        let rgb = lab.to_rgb();
        [rgb[0], rgb[1], rgb[2], (self.alpha * 255.0) as u8]
    }
}

// Types that implement Into<LabValue> also implement the Delta trait
impl From<Lab> for deltae::LabValue {
    fn from(lab: Lab) -> Self {
        Self {
            l: lab.l,
            a: lab.a,
            b: lab.b,
        }
    }
}

// Implement DeltaEq for Lab
impl<D: deltae::Delta + Copy> deltae::DeltaEq<D> for Lab {}

pub fn output_file_name(
    input_file_path: &PathBuf,
    color_palette: &ColorPalette,
    color_palette_variation: &[String],
) -> Result<String, Box<dyn Error>> {
    let file_stem = match input_file_path.file_stem() {
        Some(file_stem) => match file_stem.to_str() {
            Some(file_stem) => file_stem,
            None => return Err("Could not get file stem".into()),
        },
        None => return Err("Could not get file stem".into()),
    };
    // let file_extension = match input_file_path.extension() {
    //     Some(file_extension) => file_extension,
    //     None => return Err("Could not get file extension".into()),
    // };

    let mut output_file_name = String::new();

    output_file_name.push_str(file_stem);
    output_file_name.push_str(format!("_{}", color_palette).as_str());

    color_palette_variation.iter().for_each(|variation| {
        output_file_name.push_str(format!("-{}", variation).as_str());
    });

    output_file_name.push_str(".png");

    Ok(output_file_name)
}

// pub fn parse_color_palette(
//     json: &serde_json::Value,
// ) -> Result<HashMap<String, HashMap<String, u32>>, Box<dyn Error>> {
//     let mut color_palette = HashMap::new();
//
//     json.as_object().unwrap().iter().for_each(|(key, value)| {
//         // get the color palette variations
//         let palette = value
//             .as_object()
//             .unwrap()
//             .iter()
//             .map(|(key, value)| {
//                 let color = value.as_str().unwrap();
//                 let color = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap();
//                 (key.to_string(), color)
//             })
//             .collect();
//
//         color_palette.insert(key.to_string(), palette);
//     });
//
//     Ok(color_palette)
// }
pub fn parse_color_palette(
    json: &serde_json::Value,
) -> Result<HashMap<String, Vec<(String, u32)>>, Box<dyn Error>> {
    let mut color_palette = HashMap::new();

    json.as_object().unwrap().iter().for_each(|(key, value)| {
        // get the color palette variations as a vector to preserve order
        let palette: Vec<(String, u32)> = value
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                let color = value.as_str().unwrap();
                let color = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap();
                (key.to_string(), color)
            })
            .collect();

        color_palette.insert(key.to_string(), palette);
    });

    Ok(color_palette)
}

pub fn rgba_pixels_to_labs(img: &image::RgbaImage) -> Vec<Lab> {
    img.pixels()
        .map(|pixel| {
            let rgba = pixel.0;
            Lab::from_rgba(&rgba)
        })
        .collect()
}

// pub fn convert_palette_to_labs(palette: &HashMap<String, u32>) -> Vec<Lab> {
//     palette
//         .iter()
//         .map(|(_key, value)| {
//             let rgba = [(*value >> 16) as u8, (*value >> 8) as u8, *value as u8, 255];
//             Lab::from_rgba(&rgba)
//         })
//         .collect()
// }
pub fn convert_palette_to_labs(palette: &[(String, u32)]) -> Vec<Lab> {
    palette
        .iter()
        .map(|(_key, value)| {
            let rgba = [(*value >> 16) as u8, (*value >> 8) as u8, *value as u8, 255];
            Lab::from_rgba(&rgba)
        })
        .collect()
}

pub fn convert_lab_to_palette(lab: &Lab, palette: &[Lab]) -> [u8; 4] {
    let mut min_distance = std::f32::MAX;
    let mut color_match: Lab = Lab {
        l: 0.0,
        a: 0.0,
        b: 0.0,
        alpha: 0.0,
    };

    for color in palette {
        let delta = deltae::DeltaE::new(*lab, *color, deltae::DEMethod::DE2000);

        if delta.value() < &min_distance {
            min_distance = *delta.value();
            color_match = *color;
            color_match.alpha = lab.alpha;
        }
    }

    color_match.to_rgba()
}

pub fn ansi_paint_str(text: &str, color: u32) -> String {
    // Set foreground color as RGB
    // format!("\x1b[38;2;{};{};{}m{}\x1b[0m", (color >> 16) & 0xFF, (color >> 8) & 0xFF, color & 0xFF, text)

    // Set background color as RGB
    format!(
        "\x1b[48;2;{};{};{}m{}\x1b[0m",
        (color >> 16) & 0xFF,
        (color >> 8) & 0xFF,
        color & 0xFF,
        text
    )
}

// pub fn ansi_paint_palette(palette: &HashMap<String, u32>) {
//     println!("Palette has {} colors", palette.len());
//
//     for index in 0..palette.len() {
//         print!("{:>2}", index);
//     }
//     println!();
//     for (_key, value) in palette {
//         print!("{}", ansi_paint_str("  ", *value));
//     }
//     println!();
//     for (key, value) in palette {
//         println!("{}: {}", key, ansi_paint_str("  ", *value));
//     }
// }
pub fn ansi_paint_palette(palette: &[(String, u32)]) {
    for (index, (_key, value)) in palette.iter().enumerate() {
        print!("{}", ansi_paint_str("  ", *value));
        if index % 8 == 7 {
            println!();
        }
    }
    println!();
}

pub fn init_color_palettes() -> HashMap<String, HashMap<String, Vec<(String, u32)>>> {
    let mut color_palettes = HashMap::new();

    // catppuccin
    let json_color_palette: serde_json::Value =
        serde_json::from_str(include_str!("palettes/catppuccin.json")).unwrap();
    let color_palette = parse_color_palette(&json_color_palette).unwrap();
    color_palettes.insert("catppuccin".to_string(), color_palette);

    // gruvbox
    let json_color_palette: serde_json::Value =
        serde_json::from_str(include_str!("palettes/gruvbox.json")).unwrap();
    let color_palette = parse_color_palette(&json_color_palette).unwrap();
    color_palettes.insert("gruvbox".to_string(), color_palette);

    // gruvbox-material
    let json_color_palette: serde_json::Value =
        serde_json::from_str(include_str!("palettes/gruvbox-material.json")).unwrap();
    let color_palette = parse_color_palette(&json_color_palette).unwrap();
    color_palettes.insert("gruvbox-material".to_string(), color_palette);

    // nord
    let json_color_palette: serde_json::Value =
        serde_json::from_str(include_str!("palettes/nord.json")).unwrap();
    let color_palette = parse_color_palette(&json_color_palette).unwrap();
    color_palettes.insert("nord".to_string(), color_palette);

    color_palettes
}

pub fn get_color_palette_variations(
    color_palette: &HashMap<String, Vec<(String, u32)>>,
    variations: &[String],
) -> Vec<(String, u32)> {
    let mut palette = Vec::new();

    for variation in variations {
        let color = match color_palette.get(variation) {
            Some(color) => color,
            None => {
                eprintln!("Invalid color variation: {}", variation);
                std::process::exit(1);
            }
        };
        palette.append(&mut color.clone());
    }

    palette
}
