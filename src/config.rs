use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;

use crate::patterns::PatternSpec;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub patterns: Vec<PatternConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PatternConfig {
    pub name: String,
    pub regex: String,
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
            Ok(PatternSpec::new(&pattern.name, &pattern.regex))
        })
        .collect()
}

pub fn load_custom_patterns(config_dir: Option<&Path>) -> Result<Vec<PatternSpec>> {
    let Some(config_dir) = config_dir else {
        return Ok(Vec::new());
    };
    let config_path = config_dir.join("config.toml");
    let input = match std::fs::read_to_string(&config_path) {
        Ok(input) => input,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", config_path.display()));
        }
    };
    let config = parse_config(&input)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;
    compile_custom_patterns(&config)
}

#[cfg(test)]
mod tests {
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
}
