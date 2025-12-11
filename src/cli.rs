use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use clap::Parser;
use serde_json::Value;

use crate::delta::CLIDEMethod;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    // Options
    /// The color palette variation(s) to use
    /// Run with --help instead of -h for a list of all possible values
    ///
    /// Possible values:
    ///     - `all` to generate an image for each of the variations
    ///     - `none` if you are using a flat theme without variations
    ///     - or a comma-delimited list of the names of variations it should use
    #[arg(
        short,
        long,
        value_name = "VARIATIONS",
        default_value = "all",
        verbatim_doc_comment
    )]
    pub styles: ColorPaletteStyles,

    /// Output image(s) name/path as a comma-delimited list.
    /// Use `-` to write to stdout
    #[arg(short, long, value_name = "PATH", value_delimiter = ',')]
    pub output: Option<Vec<PathBuf>>,

    /// Output directory name/path
    #[arg(short, long, value_name = "PATH")]
    pub dir_output: Option<PathBuf>,

    /// CIELAB DeltaE method to use
    #[arg(short, long, value_enum, default_value = "de2000")]
    pub method: CLIDEMethod,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    // Arguments
    /// The color palette to use:
    ///     - name of a builtin theme
    ///     - path to a theme in JSON
    ///     - a JSON string with the theme (starting with `JSON: {}`)
    /// Run with --help instead of -h for a list of all builtin themes
    ///
    /// Builtin themes:
    ///     - catppuccin
    ///     - dracula
    ///     - edge
    ///     - everforest
    ///     - gruvbox
    ///     - gruvbox-material
    ///     - nord
    ///     - onedark
    ///     - rose-pine
    ///     - solarized
    ///     - tokyo-night
    #[arg(value_name = "PALETTE", verbatim_doc_comment)]
    pub color_palette: ColorPalette,

    /// The image(s) to process.
    /// Use `-` to read from stdin
    #[arg(value_name = "FILE", value_delimiter = ',')]
    pub process: Vec<PathBuf>,
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
    RawJSON { map: serde_json::Map<String, Value> },
    Catppuccin,
    Dracula,
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
            ColorPalette::RawJSON { map } => {
                write!(f, "JSON: {}", serde_json::to_string(map).unwrap())
            }
            ColorPalette::Catppuccin => write!(f, "catppuccin"),
            ColorPalette::Dracula => write!(f, "dracula"),
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
        if s.starts_with("JSON: ") {
            let jsonstr = &s[5..];
            let json: Value = serde_json::from_str(jsonstr).map_err(|err| err.to_string())?;
            let Value::Object(map) = json else {
                return Err("Encountered error while parsing inline JSON string: the string appears to not be a JSON object".to_string());
            };
            return Ok(ColorPalette::RawJSON { map });
        };

        let palette = match s {
            "catppuccin" => ColorPalette::Catppuccin,
            "dracula" => ColorPalette::Dracula,
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

            // The color palette seems to be the path to an external file
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
                    return Err("Encountered error while parsing JSON theme file: the contents of the file are valid JSON but do not appear to be a JSON object".to_string());
                };
                ColorPalette::RawJSON { map }
            }
        };
        Ok(palette)
    }
}
