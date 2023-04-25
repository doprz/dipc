use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use clap::{Args, Parser};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    // Options
    /// The color palette variation(s) to use
    /// Run with --help instead of -h for a list of all possible values
    ///
    /// Possible values:
    /// - `all` to generate an image for each of the variations
    /// - `none` if you are using a flat theme without variations
    /// - or a comma-delimited list of the names of variations it should use
    #[arg(
        short,
        long,
        value_name = "VARIATIONS",
        default_value = "all",
        verbatim_doc_comment
    )]
    pub styles: ColorPaletteStyles,

    /// Output directory
    #[arg(short, long, value_name = "PATH", default_value = "output")]
    pub output: PathBuf,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    // Arguments
    #[command(flatten)]
    pub color_palette: ColorPaletteArgGroup,

    /// The image(s) to process
    #[arg(value_name = "FILE")]
    pub process: Vec<PathBuf>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct ColorPaletteArgGroup {
    /// The color palette to use
    /// Run with --help instead of -h for a list of all builtin themes
    ///
    /// Builtin themes:
    /// - catppuccin
    /// - edge
    /// - everforest
    /// - gruvbox
    /// - gruvbox-material
    /// - nord
    /// - onedark
    /// - rose-pine
    /// - solarized
    /// - tokyo-night
    #[arg(short, long, value_enum, verbatim_doc_comment)]
    pub color_palette: ColorPalette,

    /// The path to a JSON file with the color palette
    #[arg(short, long)]
    pub json_file: PathBuf,

    /// A JSON string with the color palette (starting with `JSON: {}`)
    #[arg(short, long)]
    pub raw_json: String,
}

#[derive(Clone, Debug)]
pub enum ColorPaletteStyles {
    All,
    Some { styles: Vec<String> },
    None,
}

impl FromStr for ColorPaletteStyles {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let style = match s {
            "all" | "ALL" => Self::All,
            "none" | "NONE" | "no" | "NO" => Self::None,
            some => Self::Some {
                styles: {
                    let mut vars = Vec::new();
                    for var in some.split(',') {
                        if var.is_empty() {
                            return Err("One of the variations seems to be an empty string. Do you have a double comma in your variations list (-v)?".to_string());
                        };
                        vars.push(var.to_string())
                    }
                    if vars.is_empty() {
                        return Err("No styles selected".to_string());
                    };
                    vars
                },
            },
        };
        Ok(style)
    }
}

#[derive(Clone, Debug)]
pub enum ColorPalette {
    // RawJSON { map: serde_json::Map<String, Value> },
    Catppuccin,
    Edge,
    Everforest,
    Gruvbox,
    GruvboxMaterial,
    Nord,
    OneDark,
    RosePine,
    Solarized,
    TokyoNight,
}

impl std::fmt::Display for ColorPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ColorPalette::RawJSON { map } => {
            //     write!(f, "JSON: {}", serde_json::to_string(map).unwrap())
            // }
            ColorPalette::Catppuccin => write!(f, "catppuccin"),
            ColorPalette::Edge => write!(f, "edge"),
            ColorPalette::Everforest => write!(f, "everforest"),
            ColorPalette::Gruvbox => write!(f, "gruvbox"),
            ColorPalette::GruvboxMaterial => write!(f, "gruvbox-material"),
            ColorPalette::Nord => write!(f, "nord"),
            ColorPalette::OneDark => write!(f, "onedark"),
            ColorPalette::RosePine => write!(f, "rose-pine"),
            ColorPalette::Solarized => write!(f, "solarized"),
            ColorPalette::TokyoNight => write!(f, "tokyo-night"),
        }
    }
}

impl FromStr for ColorPalette {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO
        // if s.starts_with("JSON: ") {
        //     let jsonstr = &s[5..];
        //     let json: Value = serde_json::from_str(jsonstr).map_err(|err| err.to_string())?;
        //     let Value::Object(map) = json else {
        //         return Err(format!("Encountered error while parsing inline JSON string: the string appears to not be a JSON object"))
        //     };
        //     return Ok(ColorPalette::RawJSON { map });
        // };
        let palette = match s {
            "catppuccin" => ColorPalette::Catppuccin,
            "edge" => ColorPalette::Edge,
            "everforest" => ColorPalette::Everforest,
            "gruvbox" => ColorPalette::Gruvbox,
            "gruvbox-material" | "gruvbox_material" | "gruvboxmaterial" => {
                ColorPalette::GruvboxMaterial
            }
            "nord" => ColorPalette::Nord,
            "onedark" | "one_dark" | "one-dark" => ColorPalette::OneDark,
            "rose-pine" | "rose_pine" | "rosepine" => ColorPalette::RosePine,
            "solarized" => ColorPalette::Solarized,
            "tokyo-night" | "tokyo_night" | "tokyonight" => ColorPalette::TokyoNight,
            // TODO
            // The color palette seems to be the path to an external file
            // external => {
            //     let external: PathBuf = external.into();
            //     if !external.is_file() {
            //         return Err(format!("Theme source file `{s}` appears to not be a file."));
            //     };
            //     let file = File::open(external).map_err(|err| err.to_string())?;
            //     let file = BufReader::new(file);
            //     let json = serde_json::from_reader(file)
            //         .map_err(|err| format!("Error while parsing JSON content of {s}: {err}"))?;
            //     let Value::Object(map) = json else {
            //     return Err(format!("Encountered error while parsing JSON theme file: the contents of the file are valid JSON but do not appear to be a JSON object"))
            // };
            //     ColorPalette::RawJSON { map }
            // }
        };
        Ok(palette)
    }
}
