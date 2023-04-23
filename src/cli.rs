use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use clap::Parser;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The color pallete to use. The name of a builtin theme, or the path to a theme in JSON
    /// or a JSON string with the theme(starting with `JSON: {}`). Run with --help instead of -h
    /// for a list of all builtin themes
    ///
    /// Builtin themes:
    ///
    /// - catppuccin
    /// - everforest
    /// - gruvbox
    /// - gruvbox_material
    /// - nord
    /// - rosepine
    pub color_pallete: ColorPallete,

    /// The variations of the theme to generate images for.
    /// Possible values: `all` to generate an image for each of the variations, `none` if you are
    /// using a flat theme without variations, or a comma-delimited list of the names of variations
    /// it should use
    #[arg(short, long, value_name = "VARIATIONS", default_value = "all")]
    pub styles: ColorPalleteStyles,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Output directory
    #[arg(short, long, value_name = "PATH", default_value = "output")]
    pub output: PathBuf,

    /// The image to process
    #[arg(value_name = "FILE")]
    pub process: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum ColorPalleteStyles {
    All,
    Some { styles: Vec<String> },
    None,
}

impl FromStr for ColorPalleteStyles {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let variation = match s {
            "all" | "ALL" => Self::All,
            "none" | "NONE" | "no" | "NO" => Self::None,
            some => Self::Some {
                styles: {
                    let mut vars = Vec::new();
                    for var in some.split(',') {
                        if var.is_empty() {
                            return Err("One of the variations seems to be an empty string. Do you have a double comma in your variations list(-v)?".to_string());
                        };
                        vars.push(var.to_string())
                    }
                    if vars.is_empty() {
                        return Err("No variations selected".to_string());
                    };
                    vars
                },
            },
        };
        Ok(variation)
    }
}

#[derive(Clone, Debug)]
pub enum ColorPallete {
    RawJSON { map: serde_json::Map<String, Value> },
    Everforest,
    Gruvbox,
    GruvboxMaterial,
    Nord,
    RosePine,
    Catppucin,
}

impl FromStr for ColorPallete {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("JSON: ") {
            let jsonstr = &s[5..];
            let json: Value = serde_json::from_str(jsonstr).map_err(|err| err.to_string())?;
            let Value::Object(map) = json else {
                return Err(format!("Encountered error while parsing inline JSON string: the string appears to not be a JSON object"))
            };
            return Ok(ColorPallete::RawJSON { map });
        };
        let pallete = match s {
            "catppuccin" | "catpucin" | "catppucin" | "catpuccin" => ColorPallete::Catppucin,
            "everforest" => ColorPallete::Everforest,
            "gruvbox" => ColorPallete::Gruvbox,
            "gruvbox_material" | "gruvbox-material" | "gruvboxmaterial" => {
                ColorPallete::GruvboxMaterial
            }
            "nord" => ColorPallete::Nord,
            "rosepine" | "rose-pine" | "rose_pine" => ColorPallete::RosePine,
            // The color pallete seems to be the path to an external file
            external => {
                let external: PathBuf = external.into();
                if !external.is_file() {
                    return Err(format!("Theme source file `{s}` appears to not be a file."));
                };
                let file = File::open(external).map_err(|err| err.to_string())?;
                let file = BufReader::new(file);
                let json = serde_json::from_reader(file)
                    .map_err(|err| format!("Error while parsing JSON content of {s}: {err}"))?;
                let Value::Object(map) = json else {
                return Err(format!("Encountered error while parsing JSON theme file: the contents of the file are valid JSON but do not appear to be a JSON object"))
            };
                ColorPallete::RawJSON { map }
            }
        };
        Ok(pallete)
    }
}
