use anyhow::{bail, Result};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub text: String,
    pub full_text: String,
    pub start: Position,
    pub end: Position,
    pub full_start: Position,
    pub full_end: Position,
    pub start_index: usize,
    pub end_index: usize,
    pub full_start_index: usize,
    pub full_end_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinPattern {
    pub name: &'static str,
    pub regex: &'static str,
    pub ignore_line_breaks: bool,
}

pub const BUILTIN_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern {
        name: "ip",
        regex: r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "uuid",
        regex: r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "sha",
        regex: r"[0-9a-f]{7,128}",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "digit",
        regex: r"[0-9]{4,}",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "url",
        regex: r#"((https?://|git@|git://|ssh://|ftp://|file:///)[^\s()"']+)"#,
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "path",
        regex: r"(([\.\w\-~\$@]+)?(/[\.\w\-@]+)+/?)",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "hex",
        regex: r"(0x[0-9a-fA-F]+)",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "kubernetes",
        regex: r"(deployment.app|binding|componentstatuse|configmap|endpoint|event|limitrange|namespace|node|persistentvolumeclaim|persistentvolume|pod|podtemplate|replicationcontroller|resourcequota|secret|serviceaccount|service|mutatingwebhookconfiguration.admissionregistration.k8s.io|validatingwebhookconfiguration.admissionregistration.k8s.io|customresourcedefinition.apiextension.k8s.io|apiservice.apiregistration.k8s.io|controllerrevision.apps|daemonset.apps|deployment.apps|replicaset.apps|statefulset.apps|tokenreview.authentication.k8s.io|localsubjectaccessreview.authorization.k8s.io|selfsubjectaccessreviews.authorization.k8s.io|selfsubjectrulesreview.authorization.k8s.io|subjectaccessreview.authorization.k8s.io|horizontalpodautoscaler.autoscaling|cronjob.batch|job.batch|certificatesigningrequest.certificates.k8s.io|events.events.k8s.io|daemonset.extensions|deployment.extensions|ingress.extensions|networkpolicies.extensions|podsecuritypolicies.extensions|replicaset.extensions|networkpolicie.networking.k8s.io|poddisruptionbudget.policy|clusterrolebinding.rbac.authorization.k8s.io|clusterrole.rbac.authorization.k8s.io|rolebinding.rbac.authorization.k8s.io|role.rbac.authorization.k8s.io|storageclasse.storage.k8s.io)[[:alnum:]_#$%&+=/@-]+",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "kubernetes-pod",
        regex: r"[a-z][a-z0-9-]*[a-z0-9]-[bcdfghjklmnpqrstvwxz2456789]{5,10}-[bcdfghjklmnpqrstvwxz2456789]{5}",
        ignore_line_breaks: true,
    },
    BuiltinPattern {
        name: "git-status",
        regex: r"(modified|deleted|deleted by us|new file): +(?P<match>.+)",
        ignore_line_breaks: false,
    },
    BuiltinPattern {
        name: "git-status-branch",
        regex: r"Your branch is up to date with '(?P<match>.*)'.",
        ignore_line_breaks: false,
    },
    BuiltinPattern {
        name: "diff",
        regex: r"(---|\+\+\+) [ab]/(?P<match>.*)",
        ignore_line_breaks: false,
    },
];

#[derive(Debug)]
struct CompiledPattern {
    regex: Regex,
    ignore_line_breaks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternSpec {
    pub name: String,
    pub regex: String,
    pub ignore_line_breaks: bool,
}

impl PatternSpec {
    pub fn new(name: impl Into<String>, regex: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            regex: regex.into(),
            ignore_line_breaks: true,
        }
    }
}

impl From<&BuiltinPattern> for PatternSpec {
    fn from(pattern: &BuiltinPattern) -> Self {
        Self {
            name: pattern.name.to_string(),
            regex: pattern.regex.to_string(),
            ignore_line_breaks: pattern.ignore_line_breaks,
        }
    }
}

#[derive(Debug)]
pub struct Matcher {
    patterns: Vec<CompiledPattern>,
}

impl Matcher {
    pub fn builtin() -> Result<Self> {
        Self::new(BUILTIN_PATTERNS.iter().map(PatternSpec::from))
    }

    pub fn with_custom(custom_patterns: Vec<PatternSpec>) -> Result<Self> {
        Self::with_builtin_patterns(None, custom_patterns)
    }

    pub fn with_builtin_patterns(
        enabled_builtin_patterns: Option<&[String]>,
        custom_patterns: Vec<PatternSpec>,
    ) -> Result<Self> {
        let patterns = BUILTIN_PATTERNS
            .iter()
            .filter(|pattern| builtin_pattern_enabled(pattern.name, enabled_builtin_patterns))
            .map(PatternSpec::from)
            .chain(custom_patterns)
            .collect::<Vec<_>>();
        validate_builtin_pattern_names(enabled_builtin_patterns)?;
        Self::new(patterns)
    }

    pub fn new(patterns: impl IntoIterator<Item = PatternSpec>) -> Result<Self> {
        let patterns = patterns
            .into_iter()
            .map(|pattern| {
                Regex::new(&pattern.regex).map(|regex| CompiledPattern {
                    regex,
                    ignore_line_breaks: pattern.ignore_line_breaks,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { patterns })
    }

    pub fn find(&self, lines: &[String]) -> Vec<Match> {
        let flat_text = FlatText::from_lines(lines);
        let mut matches = Vec::new();
        for pattern in &self.patterns {
            if pattern.ignore_line_breaks {
                for captures in pattern.regex.captures_iter(&flat_text.text) {
                    if let Some(hit) = match_from_flat_captures(&flat_text, captures) {
                        matches.push(hit);
                    }
                }
            } else {
                for (row, line) in lines.iter().enumerate() {
                    for captures in pattern.regex.captures_iter(line) {
                        if let Some(hit) = match_from_line_captures(&flat_text, row, line, captures)
                        {
                            matches.push(hit);
                        }
                    }
                }
            }
        }
        matches.sort_by_key(|m| (m.full_start_index, std::cmp::Reverse(m.full_end_index)));
        dedupe_overlaps(matches)
    }
}

fn builtin_pattern_enabled(name: &str, enabled_builtin_patterns: Option<&[String]>) -> bool {
    enabled_builtin_patterns
        .map(|names| names.iter().any(|enabled_name| enabled_name == name))
        .unwrap_or(true)
}

fn validate_builtin_pattern_names(enabled_builtin_patterns: Option<&[String]>) -> Result<()> {
    let Some(enabled_builtin_patterns) = enabled_builtin_patterns else {
        return Ok(());
    };
    let unknown = enabled_builtin_patterns.iter().find(|name| {
        !BUILTIN_PATTERNS
            .iter()
            .any(|pattern| pattern.name == name.as_str())
    });
    if let Some(name) = unknown {
        bail!("unknown builtin pattern '{name}'");
    }
    Ok(())
}

struct FlatText {
    text: String,
    positions: Vec<Position>,
    line_start_indices: Vec<usize>,
}

impl FlatText {
    fn from_lines(lines: &[String]) -> Self {
        let mut text = String::new();
        let mut positions = Vec::new();
        let mut line_start_indices = Vec::with_capacity(lines.len());
        for (row, line) in lines.iter().enumerate() {
            line_start_indices.push(positions.len());
            for (col, ch) in line.chars().enumerate() {
                text.push(ch);
                positions.push(Position { row, col });
            }
        }
        Self {
            text,
            positions,
            line_start_indices,
        }
    }

    fn start_position(&self, char_index: usize) -> Option<Position> {
        self.positions.get(char_index).copied()
    }

    fn end_position(&self, char_index: usize) -> Option<Position> {
        if char_index == 0 {
            return None;
        }
        let previous = self.positions.get(char_index - 1).copied()?;
        Some(Position {
            row: previous.row,
            col: previous.col + 1,
        })
    }

    fn line_start_index(&self, row: usize) -> usize {
        self.line_start_indices
            .get(row)
            .copied()
            .unwrap_or_default()
    }
}

fn match_from_flat_captures(flat_text: &FlatText, captures: regex::Captures<'_>) -> Option<Match> {
    let full = captures.get(0)?;
    let selected = captures.name("match").unwrap_or(full);
    if selected.as_str().is_empty() {
        return None;
    }
    let start_index = byte_to_char_col(&flat_text.text, selected.start());
    let end_index = byte_to_char_col(&flat_text.text, selected.end());
    let full_start_index = byte_to_char_col(&flat_text.text, full.start());
    let full_end_index = byte_to_char_col(&flat_text.text, full.end());
    Some(Match {
        text: selected.as_str().to_string(),
        full_text: full.as_str().to_string(),
        start: flat_text.start_position(start_index)?,
        end: flat_text.end_position(end_index)?,
        full_start: flat_text.start_position(full_start_index)?,
        full_end: flat_text.end_position(full_end_index)?,
        start_index,
        end_index,
        full_start_index,
        full_end_index,
    })
}

fn match_from_line_captures(
    flat_text: &FlatText,
    row: usize,
    line: &str,
    captures: regex::Captures<'_>,
) -> Option<Match> {
    let full = captures.get(0)?;
    let selected = captures.name("match").unwrap_or(full);
    if selected.as_str().is_empty() {
        return None;
    }
    let line_start_index = flat_text.line_start_index(row);
    let start_col = byte_to_char_col(line, selected.start());
    let end_col = byte_to_char_col(line, selected.end());
    let full_start_col = byte_to_char_col(line, full.start());
    let full_end_col = byte_to_char_col(line, full.end());
    Some(Match {
        text: selected.as_str().to_string(),
        full_text: full.as_str().to_string(),
        start: Position {
            row,
            col: start_col,
        },
        end: Position { row, col: end_col },
        full_start: Position {
            row,
            col: full_start_col,
        },
        full_end: Position {
            row,
            col: full_end_col,
        },
        start_index: line_start_index + start_col,
        end_index: line_start_index + end_col,
        full_start_index: line_start_index + full_start_col,
        full_end_index: line_start_index + full_end_col,
    })
}

fn dedupe_overlaps(matches: Vec<Match>) -> Vec<Match> {
    let mut kept: Vec<Match> = Vec::new();
    'candidate: for candidate in matches {
        for existing in &kept {
            if ranges_overlap(
                candidate.full_start_index,
                candidate.full_end_index,
                existing.full_start_index,
                existing.full_end_index,
            ) {
                continue 'candidate;
            }
        }
        kept.push(candidate);
    }
    kept
}

fn ranges_overlap(a_start: usize, a_end: usize, b_start: usize, b_end: usize) -> bool {
    a_start < b_end && b_start < a_end
}

fn byte_to_char_col(line: &str, byte: usize) -> usize {
    line[..byte].chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches_for(pattern_name: &str, input: &str) -> Vec<String> {
        let pattern = BUILTIN_PATTERNS
            .iter()
            .find(|pattern| pattern.name == pattern_name)
            .expect("known pattern");
        let regex = Regex::new(pattern.regex).expect("valid regex");
        regex
            .captures_iter(input)
            .map(|captures| {
                captures
                    .name("match")
                    .or_else(|| captures.get(0))
                    .unwrap()
                    .as_str()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn ported_patterns_match_tmux_fingers_examples() {
        assert_eq!(
            matches_for("ip", "192.168.0.1\n127.0.0.1\nfoo"),
            ["192.168.0.1", "127.0.0.1"]
        );
        assert_eq!(
            matches_for("uuid", "d6f4b4ac-4b78-4d79-96a1-eb9ab72f2c59"),
            ["d6f4b4ac-4b78-4d79-96a1-eb9ab72f2c59"]
        );
        assert_eq!(
            matches_for("sha", "fc4fea27210bc0d85b74f40866e12890e3788134 fc4fea2",),
            ["fc4fea27210bc0d85b74f40866e12890e3788134", "fc4fea2"]
        );
        assert_eq!(
            matches_for("digit", "12345 67891011"),
            ["12345", "67891011"]
        );
        assert_eq!(
            matches_for("url", "https://geocities.com"),
            ["https://geocities.com"]
        );
        assert_eq!(
            matches_for(
                "path",
                "absolute /foo/bar/lol relative ./foo/bar/lol home ~/foo/bar/lol"
            ),
            ["/foo/bar/lol", "./foo/bar/lol", "~/foo/bar/lol"]
        );
        assert_eq!(
            matches_for("hex", "0xcafe 0xcaca 0xdeadbeef 0xCACA"),
            ["0xcafe", "0xcaca", "0xdeadbeef", "0xCACA"]
        );
    }

    #[test]
    fn git_patterns_copy_named_capture_only() {
        let status = "\
Your branch is up to date with 'origin/crystal-rewrite'.
        deleted:    CHANGELOG.md
        new file:   wat
        modified:   Makefile
        modified:   spec/lib/patterns_spec.cr
        modified:   src/fingers/config.cr";

        assert_eq!(
            matches_for("git-status", status),
            [
                "CHANGELOG.md",
                "wat",
                "Makefile",
                "spec/lib/patterns_spec.cr",
                "src/fingers/config.cr"
            ]
        );
        assert_eq!(
            matches_for("git-status-branch", status),
            ["origin/crystal-rewrite"]
        );
        assert_eq!(
            matches_for(
                "diff",
                "--- a/spec/lib/patterns_spec.cr\n+++ b/spec/lib/patterns_spec.cr"
            ),
            ["spec/lib/patterns_spec.cr", "spec/lib/patterns_spec.cr"]
        );
    }

    #[test]
    fn matcher_reports_capture_position_inside_full_match() {
        let lines = vec!["        modified:   src/main.rs".to_string()];
        let matcher = Matcher::builtin().unwrap();
        let hit = matcher
            .find(&lines)
            .into_iter()
            .find(|hit| hit.text == "src/main.rs")
            .unwrap();
        assert_eq!(hit.start.col, 20);
        assert_eq!(hit.full_start.col, 8);
    }

    #[test]
    fn longer_match_wins_when_patterns_overlap_at_same_start() {
        let lines = vec!["d6f4b4ac-4b78-4d79-96a1-eb9ab72f2c59".to_string()];
        let matcher = Matcher::builtin().unwrap();
        let hits = matcher.find(&lines);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, lines[0]);
    }

    #[test]
    fn custom_patterns_match_multiple_rust_regexes() {
        let matcher = Matcher::new([
            PatternSpec::new("ticket", r"PROJ-[0-9]+"),
            PatternSpec::new("env", r"env=(?P<match>[a-z0-9_-]+)"),
        ])
        .unwrap();
        let lines = vec!["deploy PROJ-123 env=staging-1".to_string()];
        let hits = matcher.find(&lines);

        assert_eq!(
            hits.into_iter().map(|hit| hit.text).collect::<Vec<_>>(),
            ["PROJ-123", "staging-1"]
        );
    }

    #[test]
    fn pane_line_breaks_are_ignored_for_urls_and_ips() {
        let matcher = Matcher::builtin().unwrap();
        let lines = vec![
            "open https://exa".to_string(),
            "mple.com from 192.".to_string(),
            "168.0.1".to_string(),
        ];
        let hits = matcher.find(&lines);
        let texts = hits.into_iter().map(|hit| hit.text).collect::<Vec<_>>();

        assert!(texts.contains(&"https://example.com".to_string()));
        assert!(texts.contains(&"192.168.0.1".to_string()));
    }

    #[test]
    fn matcher_can_enable_only_named_builtin_patterns() {
        let matcher =
            Matcher::with_builtin_patterns(Some(&["url".to_string()]), Vec::new()).unwrap();
        let lines = vec!["12345 https://example.com".to_string()];
        let hits = matcher.find(&lines);

        assert_eq!(
            hits.into_iter().map(|hit| hit.text).collect::<Vec<_>>(),
            ["https://example.com"]
        );
    }

    #[test]
    fn matcher_rejects_unknown_builtin_pattern_names() {
        let err =
            Matcher::with_builtin_patterns(Some(&["missing".to_string()]), Vec::new()).unwrap_err();

        assert!(err.to_string().contains("missing"));
    }
}
