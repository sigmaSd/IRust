use crate::irust::IRustError;
use crossterm::style::Color;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub fn theme() -> Result<Theme, IRustError> {
    let theme_file = dirs_next::config_dir()
        .ok_or("Error accessing config_dir")?
        .join("irust")
        .join("theme");

    let data = std::fs::read_to_string(theme_file)?;

    Ok(toml::from_str(&data)?)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Theme {
    pub keyword: String,
    pub keyword2: String,
    pub function: String,
    pub r#type: String,
    pub number: String,
    pub symbol: String,
    pub r#macro: String,
    pub string_literal: String,
    pub character: String,
    pub lifetime: String,
    pub comment: String,
    pub r#const: String,
    pub x: String,
}

impl Theme {
    pub fn save(&self) -> Result<(), IRustError> {
        let theme_path = dirs_next::config_dir()
            .ok_or("Error accessing config_dir")?
            .join("irust")
            .join("theme");
        let mut theme = std::fs::File::create(&theme_path)?;
        write!(theme, "{}", toml::to_string(&self)?)?;
        Ok(())
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            keyword: "magenta".into(),
            keyword2: "dark_red".into(),
            function: "blue".into(),
            r#type: "cyan".into(),
            number: "dark_yellow".into(),
            symbol: "red".into(),
            r#macro: "dark_yellow".into(),
            string_literal: "yellow".into(),
            character: "green".into(),
            lifetime: "dark_magenta".into(),
            comment: "dark_grey".into(),
            r#const: "dark_green".into(),
            x: "white".into(),
        }
    }
}

pub fn theme_color_to_term_color(color: &str) -> Option<Color> {
    if color.starts_with('#') {
        if color.len() != 7 {
            return None;
        }
        // Hex color name
        let parse = || -> Option<Color> {
            let color = &color[1..];
            let r = u8::from_str_radix(&color[0..2], 16).ok()?;
            let g = u8::from_str_radix(&color[2..4], 16).ok()?;
            let b = u8::from_str_radix(&color[4..], 16).ok()?;
            Some(Color::Rgb { r, g, b })
        };
        parse()
    } else {
        // try color as name
        try_from(color).ok()
    }
}

// To be Removed
fn try_from(src: &str) -> Result<Color, IRustError> {
    let src = src.to_lowercase();

    match src.as_ref() {
        "black" => Ok(Color::Black),
        "dark_grey" => Ok(Color::DarkGrey),
        "red" => Ok(Color::Red),
        "dark_red" => Ok(Color::DarkRed),
        "green" => Ok(Color::Green),
        "dark_green" => Ok(Color::DarkGreen),
        "yellow" => Ok(Color::Yellow),
        "dark_yellow" => Ok(Color::DarkYellow),
        "blue" => Ok(Color::Blue),
        "dark_blue" => Ok(Color::DarkBlue),
        "magenta" => Ok(Color::Magenta),
        "dark_magenta" => Ok(Color::DarkMagenta),
        "cyan" => Ok(Color::Cyan),
        "dark_cyan" => Ok(Color::DarkCyan),
        "white" => Ok(Color::White),
        "grey" => Ok(Color::Grey),
        _ => Err(IRustError::Custom("Uknown color".into())),
    }
}
