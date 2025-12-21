use std::path::PathBuf;

pub fn is_image_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        .unwrap_or(false)
}

pub fn parse_color_value(val: &serde_json::Value) -> Option<[u8; 3]> {
    match val {
        serde_json::Value::String(hex) if hex.starts_with('#') => {
            let hex = &hex[1..];
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some([r, g, b])
            } else {
                None
            }
        }
        serde_json::Value::Array(arr) if arr.len() == 3 => {
            let r = arr[0].as_u64()? as u8;
            let g = arr[1].as_u64()? as u8;
            let b = arr[2].as_u64()? as u8;
            Some([r, g, b])
        }
        _ => None,
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
