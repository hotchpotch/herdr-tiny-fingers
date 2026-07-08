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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinPattern {
    pub name: &'static str,
    pub regex: &'static str,
}

pub const BUILTIN_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern {
        name: "ip",
        regex: r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
    },
    BuiltinPattern {
        name: "uuid",
        regex: r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
    },
    BuiltinPattern {
        name: "sha",
        regex: r"[0-9a-f]{7,128}",
    },
    BuiltinPattern {
        name: "digit",
        regex: r"[0-9]{4,}",
    },
    BuiltinPattern {
        name: "url",
        regex: r#"((https?://|git@|git://|ssh://|ftp://|file:///)[^\s()"']+)"#,
    },
    BuiltinPattern {
        name: "path",
        regex: r"(([\.\w\-~\$@]+)?(/[\.\w\-@]+)+/?)",
    },
    BuiltinPattern {
        name: "hex",
        regex: r"(0x[0-9a-fA-F]+)",
    },
    BuiltinPattern {
        name: "kubernetes",
        regex: r"(deployment.app|binding|componentstatuse|configmap|endpoint|event|limitrange|namespace|node|persistentvolumeclaim|persistentvolume|pod|podtemplate|replicationcontroller|resourcequota|secret|serviceaccount|service|mutatingwebhookconfiguration.admissionregistration.k8s.io|validatingwebhookconfiguration.admissionregistration.k8s.io|customresourcedefinition.apiextension.k8s.io|apiservice.apiregistration.k8s.io|controllerrevision.apps|daemonset.apps|deployment.apps|replicaset.apps|statefulset.apps|tokenreview.authentication.k8s.io|localsubjectaccessreview.authorization.k8s.io|selfsubjectaccessreviews.authorization.k8s.io|selfsubjectrulesreview.authorization.k8s.io|subjectaccessreview.authorization.k8s.io|horizontalpodautoscaler.autoscaling|cronjob.batch|job.batch|certificatesigningrequest.certificates.k8s.io|events.events.k8s.io|daemonset.extensions|deployment.extensions|ingress.extensions|networkpolicies.extensions|podsecuritypolicies.extensions|replicaset.extensions|networkpolicie.networking.k8s.io|poddisruptionbudget.policy|clusterrolebinding.rbac.authorization.k8s.io|clusterrole.rbac.authorization.k8s.io|rolebinding.rbac.authorization.k8s.io|role.rbac.authorization.k8s.io|storageclasse.storage.k8s.io)[[:alnum:]_#$%&+=/@-]+",
    },
    BuiltinPattern {
        name: "kubernetes-pod",
        regex: r"[a-z][a-z0-9-]*[a-z0-9]-[bcdfghjklmnpqrstvwxz2456789]{5,10}-[bcdfghjklmnpqrstvwxz2456789]{5}",
    },
    BuiltinPattern {
        name: "git-status",
        regex: r"(modified|deleted|deleted by us|new file): +(?P<match>.+)",
    },
    BuiltinPattern {
        name: "git-status-branch",
        regex: r"Your branch is up to date with '(?P<match>.*)'.",
    },
    BuiltinPattern {
        name: "diff",
        regex: r"(---|\+\+\+) [ab]/(?P<match>.*)",
    },
];

#[derive(Debug)]
struct CompiledPattern {
    regex: Regex,
}

#[derive(Debug)]
pub struct Matcher {
    patterns: Vec<CompiledPattern>,
}

impl Matcher {
    pub fn builtin() -> Result<Self, regex::Error> {
        let patterns = BUILTIN_PATTERNS
            .iter()
            .map(|pattern| Regex::new(pattern.regex).map(|regex| CompiledPattern { regex }))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { patterns })
    }

    pub fn find(&self, lines: &[String]) -> Vec<Match> {
        let mut matches = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            for pattern in &self.patterns {
                for captures in pattern.regex.captures_iter(line) {
                    let Some(full) = captures.get(0) else {
                        continue;
                    };
                    let selected = captures.name("match").unwrap_or(full);
                    if selected.as_str().is_empty() {
                        continue;
                    }
                    matches.push(Match {
                        text: selected.as_str().to_string(),
                        full_text: full.as_str().to_string(),
                        start: Position {
                            row,
                            col: byte_to_char_col(line, selected.start()),
                        },
                        end: Position {
                            row,
                            col: byte_to_char_col(line, selected.end()),
                        },
                        full_start: Position {
                            row,
                            col: byte_to_char_col(line, full.start()),
                        },
                        full_end: Position {
                            row,
                            col: byte_to_char_col(line, full.end()),
                        },
                    });
                }
            }
        }
        matches.sort_by_key(|m| {
            (
                m.full_start.row,
                m.full_start.col,
                std::cmp::Reverse(m.full_end.col),
            )
        });
        dedupe_overlaps(matches)
    }
}

fn dedupe_overlaps(matches: Vec<Match>) -> Vec<Match> {
    let mut kept: Vec<Match> = Vec::new();
    'candidate: for candidate in matches {
        for existing in &kept {
            if candidate.full_start.row == existing.full_start.row
                && ranges_overlap(
                    candidate.full_start.col,
                    candidate.full_end.col,
                    existing.full_start.col,
                    existing.full_end.col,
                )
            {
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
}
