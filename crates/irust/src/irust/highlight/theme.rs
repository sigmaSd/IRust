use std::fs::DirEntry;

use crate::irust::Result;
use crossterm::style::Color;
use serde::{Deserialize, Serialize};

fn themes_path() -> Result<std::path::PathBuf> {
    Ok(dirs::config_dir()
        .ok_or("Error accessing config_dir")?
        .join("irust")
        .join("themes"))
}

pub fn theme(name: String) -> Result<Theme> {
    let selected_theme_path = themes_path()?.join(name + ".toml");

    let data = std::fs::read_to_string(selected_theme_path)?;

    Ok(toml::from_str(&data)?)
}

pub fn theme_or_create_default(name: String) -> Theme {
    let maybe_theme = theme(name);
    if let Ok(theme) = maybe_theme {
        return theme;
    }

    let _ = (|| -> Result<()> {
        std::fs::create_dir_all(themes_path()?)?;
        std::fs::write(
            themes_path()?.join("default.toml"),
            toml::to_string(&Theme::default())?,
        )?;
        Ok(())
    })();
    Theme::default()
}

pub fn installed_themes() -> Result<Vec<DirEntry>> {
    Ok(std::fs::read_dir(
        dirs::config_dir()
            .ok_or("Error accessing config_dir")?
            .join("irust")
            .join("themes"),
    )?
    .collect::<std::io::Result<Vec<_>>>()?)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Theme {
    pub keyword: String,
    pub keyword2: String,
    pub function: String,
    pub r#type: String,
    pub symbol: String,
    pub r#macro: String,
    pub literal: String,
    pub lifetime: String,
    pub comment: String,
    pub r#const: String,
    pub ident: String,
    pub paren_rainbow: bool,
}

impl Theme {
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
            symbol: "red".into(),
            r#macro: "dark_yellow".into(),
            literal: "yellow".into(),
            lifetime: "dark_magenta".into(),
            comment: "dark_grey".into(),
            r#const: "dark_green".into(),
            ident: "white".into(),
            paren_rainbow: true,
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
        // we only support lowercase for performance
        // because this is a hot path
        match color {
            "black" => Some(Color::Black),
            "dark_grey" => Some(Color::DarkGrey),
            "red" => Some(Color::Red),
            "dark_red" => Some(Color::DarkRed),
            "green" => Some(Color::Green),
            "dark_green" => Some(Color::DarkGreen),
            "yellow" => Some(Color::Yellow),
            "dark_yellow" => Some(Color::DarkYellow),
            "blue" => Some(Color::Blue),
            "dark_blue" => Some(Color::DarkBlue),
            "magenta" => Some(Color::Magenta),
            "dark_magenta" => Some(Color::DarkMagenta),
            "cyan" => Some(Color::Cyan),
            "dark_cyan" => Some(Color::DarkCyan),
            "white" => Some(Color::White),
            "grey" => Some(Color::Grey),
            _ => None,
        }
    }
}
