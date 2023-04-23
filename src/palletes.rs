use serde_json::Value;

use crate::cli::ColorPallete;

impl ColorPallete {
    pub fn get_json(self) -> serde_json::Map<String, Value> {
        let colors = match self {
            #[rustfmt::skip]
            ColorPallete::Nord => serde_json::from_str(include_str!("./palettes/nord.json")).unwrap(),
            ColorPallete::RosePine => {
                serde_json::from_str(include_str!("./palettes/rose-pine.json")).unwrap()
            }
            ColorPallete::Catppucin => {
                serde_json::from_str(include_str!("./palettes/catppuccin.json")).unwrap()
            }
            ColorPallete::Everforest => {
                serde_json::from_str(include_str!("./palettes/everforest.json")).unwrap()
            }
            ColorPallete::GruvboxMaterial => {
                serde_json::from_str(include_str!("./palettes/gruvbox-material.json")).unwrap()
            }
            ColorPallete::Gruvbox => {
                serde_json::from_str(include_str!("./palettes/gruvbox.json")).unwrap()
            }
            ColorPallete::Edge => {
                serde_json::from_str(include_str!("./palettes/edge.json")).unwrap()
            }
            ColorPallete::TokyoNight => {
                serde_json::from_str(include_str!("./palettes/tokyo-night.json")).unwrap()
            }
            ColorPallete::RawJSON { map } => return map,
            // _ => todo!(),
        };
        let Value::Object(obj) = colors else {
            panic!("An included theme appears to not be a JSON object?")
        };
        obj
    }
}
