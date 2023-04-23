use serde_json::Value;

use crate::cli::ColorPalette;

impl ColorPalette {
    pub fn get_json(self) -> serde_json::Map<String, Value> {
        let colors = match self {
            ColorPalette::Nord => {
                serde_json::from_str(include_str!("./palettes/nord.json")).unwrap()
            }
            ColorPalette::RosePine => {
                serde_json::from_str(include_str!("./palettes/rose-pine.json")).unwrap()
            }
            ColorPalette::Catppucin => {
                serde_json::from_str(include_str!("./palettes/catppuccin.json")).unwrap()
            }
            ColorPalette::Everforest => {
                serde_json::from_str(include_str!("./palettes/everforest.json")).unwrap()
            }
            ColorPalette::GruvboxMaterial => {
                serde_json::from_str(include_str!("./palettes/gruvbox-material.json")).unwrap()
            }
            ColorPalette::Gruvbox => {
                serde_json::from_str(include_str!("./palettes/gruvbox.json")).unwrap()
            }
            ColorPalette::Edge => {
                serde_json::from_str(include_str!("./palettes/edge.json")).unwrap()
            }
            ColorPalette::TokyoNight => {
                serde_json::from_str(include_str!("./palettes/tokyo-night.json")).unwrap()
            }
            ColorPalette::RawJSON { map } => return map,
        };
        let Value::Object(obj) = colors else {
            panic!("An included theme appears to not be a JSON object?")
        };
        obj
    }
}
