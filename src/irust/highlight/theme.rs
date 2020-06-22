use crossterm::style::Color;
use std::collections::HashMap;

pub fn theme() -> Option<HashMap<String, Color>> {
    let theme_file = dirs_next::config_dir()?.join("irust").join("theme");

    let data = std::fs::read_to_string(theme_file).ok()?;

    let mut map = HashMap::new();

    for line in data.lines() {
        let mut line = line.split(':');
        let key = line.next()?.trim();
        let mut color = line.next()?.trim().to_string();
        let (r, g, b) = if color.starts_with('#') {
            // Hex color name
            color.remove(0);
            let r = u8::from_str_radix(&color[0..2], 16).ok()?;
            let g = u8::from_str_radix(&color[2..4], 16).ok()?;
            let b = u8::from_str_radix(&color[4..], 16).ok()?;
            (r, g, b)
        } else {
            todo!()
        };
        map.insert(key.to_string(), Color::Rgb { r, g, b });
    }

    Some(map)
}
