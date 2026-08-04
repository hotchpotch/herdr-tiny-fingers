use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;

use crate::patterns::PatternSpec;
use crate::theme::{parse_color, Theme};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub enabled_builtin_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub copy_toast: bool,
    #[serde(default)]
    pub direct_paste: bool,
    pub style: Option<StyleConfig>,
    #[serde(default)]
    pub patterns: Vec<PatternConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PatternConfig {
    pub name: String,
    pub regex: String,
    #[serde(default = "default_ignore_line_breaks")]
    pub ignore_line_breaks: bool,
}

fn default_ignore_line_breaks() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StyleConfig {
    pub hint_fg: Option<String>,
    pub hint_bg: Option<String>,
    pub match_fg: Option<String>,
    pub match_bg: Option<String>,
    pub selected_hint_fg: Option<String>,
    pub selected_hint_bg: Option<String>,
    pub selected_match_fg: Option<String>,
    pub selected_match_bg: Option<String>,
    pub status_fg: Option<String>,
    pub status_bg: Option<String>,
    pub empty_fg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternSettings {
    pub enabled_builtin_patterns: Option<Vec<String>>,
    pub custom_patterns: Vec<PatternSpec>,
    pub theme: Theme,
    pub copy_toast: bool,
    pub direct_paste: bool,
}

pub fn parse_config(input: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(input)
}

pub fn compile_custom_patterns(config: &Config) -> Result<Vec<PatternSpec>> {
    config
        .patterns
        .iter()
        .map(|pattern| {
            Regex::new(&pattern.regex)
                .with_context(|| format!("invalid custom regex pattern '{}'", pattern.name))?;
            let mut spec = PatternSpec::new(&pattern.name, &pattern.regex);
            spec.ignore_line_breaks = pattern.ignore_line_breaks;
            Ok(spec)
        })
        .collect()
}

pub fn compile_pattern_settings(config: &Config) -> Result<PatternSettings> {
    Ok(PatternSettings {
        enabled_builtin_patterns: config.enabled_builtin_patterns.clone(),
        custom_patterns: compile_custom_patterns(config)?,
        theme: compile_theme(config.style.as_ref())?,
        copy_toast: config.copy_toast,
        direct_paste: config.direct_paste,
    })
}

pub fn compile_theme(style: Option<&StyleConfig>) -> Result<Theme> {
    let mut theme = Theme::default();
    let Some(style) = style else {
        return Ok(theme);
    };
    if let Some(value) = &style.hint_fg {
        theme.hint_fg = parse_color(value).with_context(|| "invalid style.hint_fg")?;
    }
    if let Some(value) = &style.hint_bg {
        theme.hint_bg = parse_color(value).with_context(|| "invalid style.hint_bg")?;
    }
    if let Some(value) = &style.match_fg {
        theme.match_fg = parse_color(value).with_context(|| "invalid style.match_fg")?;
    }
    if let Some(value) = &style.match_bg {
        theme.match_bg = Some(parse_color(value).with_context(|| "invalid style.match_bg")?);
    }
    if let Some(value) = &style.selected_hint_fg {
        theme.selected_hint_fg =
            parse_color(value).with_context(|| "invalid style.selected_hint_fg")?;
    }
    if let Some(value) = &style.selected_hint_bg {
        theme.selected_hint_bg =
            parse_color(value).with_context(|| "invalid style.selected_hint_bg")?;
    }
    if let Some(value) = &style.selected_match_fg {
        theme.selected_match_fg =
            parse_color(value).with_context(|| "invalid style.selected_match_fg")?;
    }
    if let Some(value) = &style.selected_match_bg {
        theme.selected_match_bg =
            parse_color(value).with_context(|| "invalid style.selected_match_bg")?;
    }
    if let Some(value) = &style.status_fg {
        theme.status_fg = parse_color(value).with_context(|| "invalid style.status_fg")?;
    }
    if let Some(value) = &style.status_bg {
        theme.status_bg = parse_color(value).with_context(|| "invalid style.status_bg")?;
    }
    if let Some(value) = &style.empty_fg {
        theme.empty_fg = parse_color(value).with_context(|| "invalid style.empty_fg")?;
    }
    Ok(theme)
}

pub fn load_custom_patterns(config_dir: Option<&Path>) -> Result<Vec<PatternSpec>> {
    Ok(load_pattern_settings(config_dir)?.custom_patterns)
}

pub fn load_pattern_settings(config_dir: Option<&Path>) -> Result<PatternSettings> {
    let Some(config_dir) = config_dir else {
        return Ok(PatternSettings {
            enabled_builtin_patterns: None,
            custom_patterns: Vec::new(),
            theme: Theme::default(),
            copy_toast: false,
            direct_paste: false,
        });
    };
    let config_path = config_dir.join("config.toml");
    let input = match std::fs::read_to_string(&config_path) {
        Ok(input) => input,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PatternSettings {
                enabled_builtin_patterns: None,
                custom_patterns: Vec::new(),
                theme: Theme::default(),
                copy_toast: false,
                direct_paste: false,
            });
        }
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", config_path.display()));
        }
    };
    let config = parse_config(&input)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;
    compile_pattern_settings(&config)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::*;

    #[test]
    fn parses_multiple_custom_patterns() {
        let config = parse_config(
            r#"
[[patterns]]
name = "ticket"
regex = "PROJ-[0-9]+"

[[patterns]]
name = "env"
regex = "env=(?P<match>[a-z0-9_-]+)"
"#,
        )
        .unwrap();

        assert_eq!(config.patterns.len(), 2);
        assert_eq!(config.patterns[0].name, "ticket");
        assert_eq!(config.patterns[1].regex, "env=(?P<match>[a-z0-9_-]+)");
    }

    #[test]
    fn custom_pattern_line_break_handling_defaults_to_true_and_can_be_disabled() {
        let defaults = parse_config(
            r#"
[[patterns]]
name = "ticket"
regex = "PROJ-[0-9]+"
"#,
        )
        .unwrap();
        assert!(defaults.patterns[0].ignore_line_breaks);

        let line_by_line = parse_config(
            r#"
[[patterns]]
name = "k8s"
regex = "(?m)^\\S+"
ignore_line_breaks = false
"#,
        )
        .unwrap();
        assert!(!line_by_line.patterns[0].ignore_line_breaks);

        let settings = compile_pattern_settings(&line_by_line).unwrap();
        assert!(!settings.custom_patterns[0].ignore_line_breaks);
    }

    #[test]
    fn rejects_invalid_custom_regex() {
        let config = parse_config(
            r#"
[[patterns]]
name = "bad"
regex = "["
"#,
        )
        .unwrap();

        let err = compile_custom_patterns(&config).unwrap_err();
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn parses_enabled_builtin_pattern_names() {
        let config = parse_config(
            r#"
enabled_builtin_patterns = ["url", "sha"]

[[patterns]]
name = "ticket"
regex = "PROJ-[0-9]+"
"#,
        )
        .unwrap();

        assert_eq!(
            config.enabled_builtin_patterns,
            Some(vec!["url".to_string(), "sha".to_string()])
        );
    }

    #[test]
    fn copy_toast_defaults_to_false() {
        let config = parse_config("").unwrap();
        let settings = compile_pattern_settings(&config).unwrap();

        assert!(!config.copy_toast);
        assert!(!settings.copy_toast);
    }

    #[test]
    fn parses_copy_toast_true() {
        let config = parse_config("copy_toast = true").unwrap();
        let settings = compile_pattern_settings(&config).unwrap();

        assert!(config.copy_toast);
        assert!(settings.copy_toast);
    }

    #[test]
    fn direct_paste_defaults_to_false() {
        let config = parse_config("").unwrap();
        let settings = compile_pattern_settings(&config).unwrap();

        assert!(!config.direct_paste);
        assert!(!settings.direct_paste);
    }

    #[test]
    fn parses_direct_paste_true() {
        let config = parse_config("direct_paste = true").unwrap();
        let settings = compile_pattern_settings(&config).unwrap();

        assert!(config.direct_paste);
        assert!(settings.direct_paste);
    }

    #[test]
    fn compiles_pattern_settings_with_builtin_selection() {
        let config = parse_config(
            r#"
enabled_builtin_patterns = ["url"]

[[patterns]]
name = "ticket"
regex = "PROJ-[0-9]+"
"#,
        )
        .unwrap();

        let settings = compile_pattern_settings(&config).unwrap();
        assert_eq!(
            settings.enabled_builtin_patterns,
            Some(vec!["url".to_string()])
        );
        assert_eq!(settings.custom_patterns.len(), 1);
    }

    #[test]
    fn compiles_custom_style_colors() {
        let config = parse_config(
            r##"
[style]
hint_fg = "black"
hint_bg = "light-yellow"
match_fg = "#ffd75f"
match_bg = "dark-gray"
selected_hint_bg = "magenta"
status_bg = "blue"
"##,
        )
        .unwrap();

        let settings = compile_pattern_settings(&config).unwrap();

        assert_eq!(settings.theme.hint_bg, Color::LightYellow);
        assert_eq!(settings.theme.match_fg, Color::Rgb(255, 215, 95));
        assert_eq!(settings.theme.match_bg, Some(Color::DarkGray));
        assert_eq!(settings.theme.status_bg, Color::Blue);
    }

    #[test]
    fn rejects_invalid_style_colors() {
        let config = parse_config(
            r#"
[style]
hint_bg = "not-a-color"
"#,
        )
        .unwrap();

        let err = compile_pattern_settings(&config).unwrap_err();

        assert!(err.to_string().contains("style.hint_bg"));
    }
}
