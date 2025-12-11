use std::path::Path;
use std::path::PathBuf;

use image::Rgb;
use serde_json::Value;

use crate::cli::{ColorPalette, ColorPaletteStyles};

pub fn parse_palette(
    json: serde_json::Map<String, Value>,
    styles: &ColorPaletteStyles,
) -> Result<Vec<Palette>, String> {
    match styles {
        ColorPaletteStyles::None => {
            // Flat theme
            Ok(vec![Palette::try_from(json)?])
        }
        ColorPaletteStyles::All => {
            // Parse all styles
            let mut out = Vec::with_capacity(json.len());
            for (style, val) in json {
                let Value::Object(map) = val else {
                    return Err(format!(
                        "Failed to parse palette style `{style}`: It's value is not a JSON object"
                    ));
                };
                let mut palette = Palette::try_from(map)
                    .map_err(|err| format!("Failed to parse palette style `{style}`: {err}"))?;
                palette.name = Some(style);
                out.push(palette);
            }
            Ok(out)
        }
        ColorPaletteStyles::Some { styles } => {
            let mut json = json;
            let mut out = Vec::with_capacity(styles.len());
            for style in styles {
                let Some(Value::Object(map)) = json.remove(style) else {
                    return Err(format!("Failed to parse palette style `{style}`: It does not exist in the theme JSON source"));
                };
                let mut palette = Palette::try_from(map)
                    .map_err(|err| format!("Failed to parse palette style `{style}`: {err}"))?;
                palette.name = Some(style.to_string());
                out.push(palette);
            }
            Ok(out)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub name: Option<String>,
    pub colors: Vec<(String, Rgb<u8>)>,
}

impl TryFrom<serde_json::Map<String, Value>> for Palette {
    type Error = String;

    fn try_from(json: serde_json::Map<String, Value>) -> Result<Self, Self::Error> {
        let mut colors = Vec::with_capacity(json.len());
        for (name, value) in json {
            let mut colorarr: [u8; 3] = [0_u8; 3];
            match value {
                Value::String(hex) => {
                    // For representing a color as a hex string `#FF8800` in JSON
                    if !hex.starts_with('#') {
                        return Err(format!(
                            "Encountered a color string not in the `#HEX` format: `{hex}`"
                        ));
                    };
                    let color = &hex[1..];
                    if !matches!(color.len(), 3 | 6) {
                        return Err(format!(
                            "Encountered a HEX color string of an invalid length: `{hex}`"
                        ));
                    }
                    let channel_length = color.len() / 3;
                    let multiplier = match channel_length {
                        1 => 16,
                        2 => 1,
                        _ => unreachable!(),
                    };
                    for (channel, c) in colorarr.iter_mut().enumerate() {
                        let start = channel * channel_length;
                        let Some(channelstr) = color.get(start..start + channel_length) else {
                            return Err(format!(
                                "Failed to parse HEX color string `{hex}`. Does it contain a multi-byte sequence? Only hexadecimal digits are allowed."
                            ));
                        };
                        let Ok(val) = u8::from_str_radix(channelstr, 16).map(|x| x * multiplier)
                        else {
                            return Err(format!(
                                "Failed to parse HEX color string `{hex}`. Only hexadecimal digits are allowed."
                            ));
                        };
                        *c = val;
                    }
                }
                Value::Array(arr) => {
                    // For representing a color as `[128, 255, 0]` in JSON
                    if arr.len() != 3 {
                        return Err(format!(
                            "Encountered a color array with {} elements instead of 3: {arr:?}",
                            arr.len()
                        ));
                    }
                    for (i, channel) in arr.iter().enumerate() {
                        let Value::Number(num) = channel else {
                            return Err(format!(
                                "Encountered a non-number in a color array: {arr:?}"
                            ));
                        };
                        let Some(Ok(brightness)): Option<Result<u8, _>> =
                            num.as_u64().map(|num| num.try_into())
                        else {
                            return Err(format!("Encountered a number not representable by an 8-bit-integer in a color array: {arr:?}, element {i}"));
                        };
                        colorarr[i] = brightness
                    }
                }
                Value::Object(mut map) => {
                    // For representing a color as a JSON object: `{"r": 255, "g": 128, "b": 0}`
                    for (channel, name) in ["r", "g", "b"].into_iter().enumerate() {
                        let Some(obj) = map.remove(name) else {
                            return Err(format!(
                                r#"Key `{name}` not found in JSON object {map:?}. The format is `{{"r": 255, "g": 128, "b": 0\}}"#
                            ));
                        };
                        let Value::Number(num) = obj else {
                            return Err(format!(
                                r#"Key `{name}` has a non-number value in JSON object {map:?}. The format is `{{"r": 255, "g": 128, "b": 0}}"#
                            ));
                        };
                        let Some(Ok(brightness)): Option<Result<u8, _>> =
                            num.as_u64().map(|num| num.try_into())
                        else {
                            return Err(format!("Encountered a number not representable by an 8-bit-integer in a color object: at key {name}: {num}"));
                        };
                        colorarr[channel] = brightness;
                    }
                }
                _ => {}
            };
            colors.push((name, Rgb(colorarr)))
        }
        Ok(Palette { colors, name: None })
    }
}

pub fn output_file_name(
    dir_path: &Option<PathBuf>,
    input_path: &Path,
    color_palette: &ColorPalette,
    color_palette_variations: &[Palette],
    method: deltae::DEMethod,
) -> PathBuf {
    let mut output = PathBuf::new();
    let mut output_file_name = String::new();

    if let Some(dir) = dir_path {
        match dir.to_str() {
            Some(dir) => output.push(dir),
            None => {
                eprintln!("Failed to convert directory path to string");
                std::process::exit(1);
            }
        };
    }

    let file_stem = match input_path.file_stem() {
        Some(stem) => match stem.to_str() {
            Some(stem) => stem,
            None => {
                eprintln!("Failed to convert file stem to string");
                eprintln!("Defaulting to \"image\"");
                "image"
            }
        },
        None => {
            eprintln!("Failed to get file stem");
            eprintln!("Defaulting to \"image\"");
            "image"
        }
    };
    output_file_name.push_str(file_stem);

    let color_palette: String = match &color_palette {
        ColorPalette::RawJSON { .. } => String::from("custom"),
        _ => format!("{color_palette}"),
    };
    output_file_name.push_str(format!("_{color_palette}").as_str());

    color_palette_variations.iter().for_each(|variation| {
        if let Some(name) = &variation.name {
            output_file_name.push_str(format!("-{}", name.replace(' ', "_")).as_str());
        }
    });

    if method != deltae::DEMethod::DE2000 {
        output_file_name.push_str(format!("_{method}").as_str());
    }

    output.push(output_file_name);
    output.set_extension("png");
    output
}
