use anyhow::{bail, Result};
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub hint_fg: Color,
    pub hint_bg: Color,
    pub match_fg: Color,
    pub match_bg: Option<Color>,
    pub selected_hint_fg: Color,
    pub selected_hint_bg: Color,
    pub selected_match_fg: Color,
    pub selected_match_bg: Color,
    pub status_fg: Color,
    pub status_bg: Color,
    pub empty_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            hint_fg: Color::Black,
            hint_bg: Color::Yellow,
            match_fg: Color::Yellow,
            match_bg: None,
            selected_hint_fg: Color::White,
            selected_hint_bg: Color::Magenta,
            selected_match_fg: Color::White,
            selected_match_bg: Color::Magenta,
            status_fg: Color::Black,
            status_bg: Color::Gray,
            empty_fg: Color::Yellow,
        }
    }
}

impl Theme {
    pub fn hint_style(&self, selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(self.selected_hint_fg)
                .bg(self.selected_hint_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(self.hint_fg)
                .bg(self.hint_bg)
                .add_modifier(Modifier::BOLD)
        }
    }

    pub fn match_style(&self, selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(self.selected_match_fg)
                .bg(self.selected_match_bg)
        } else {
            style_with_optional_bg(self.match_fg, self.match_bg)
        }
    }

    pub fn status_style(&self) -> Style {
        Style::default().fg(self.status_fg).bg(self.status_bg)
    }

    pub fn empty_style(&self) -> Style {
        Style::default().fg(self.empty_fg)
    }
}

fn style_with_optional_bg(fg: Color, bg: Option<Color>) -> Style {
    let style = Style::default().fg(fg);
    if let Some(bg) = bg {
        style.bg(bg)
    } else {
        style
    }
}

pub fn parse_color(input: &str) -> Result<Color> {
    let normalized = input.trim().to_ascii_lowercase();
    let color = match normalized.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "dark-gray" | "dark-grey" => Color::DarkGray,
        "light-red" => Color::LightRed,
        "light-green" => Color::LightGreen,
        "light-yellow" => Color::LightYellow,
        "light-blue" => Color::LightBlue,
        "light-magenta" => Color::LightMagenta,
        "light-cyan" => Color::LightCyan,
        "white" => Color::White,
        _ if normalized.starts_with('#') => parse_hex_color(&normalized)?,
        _ => bail!("unknown color '{input}'"),
    };
    Ok(color)
}

fn parse_hex_color(input: &str) -> Result<Color> {
    let hex = input.trim_start_matches('#');
    if hex.len() != 6 {
        bail!("hex colors must use #RRGGBB, got '{input}'");
    }
    let red = u8::from_str_radix(&hex[0..2], 16)?;
    let green = u8::from_str_radix(&hex[2..4], 16)?;
    let blue = u8::from_str_radix(&hex[4..6], 16)?;
    Ok(Color::Rgb(red, green, blue))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_uses_less_harsh_match_colors() {
        let theme = Theme::default();

        assert_eq!(theme.hint_bg, Color::Yellow);
        assert_eq!(theme.match_fg, Color::Yellow);
        assert_eq!(theme.match_bg, None);
        assert_eq!(theme.selected_hint_bg, Color::Magenta);
    }

    #[test]
    fn parses_named_and_hex_colors() {
        assert_eq!(parse_color("light-blue").unwrap(), Color::LightBlue);
        assert_eq!(parse_color("#112233").unwrap(), Color::Rgb(17, 34, 51));
    }

    #[test]
    fn rejects_unknown_colors() {
        assert!(parse_color("not-a-color").is_err());
    }
}
