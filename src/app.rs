use crate::hints::{assign_hints, HintTarget};
use crate::patterns::{Match, Matcher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Copy(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct App {
    pub lines: Vec<String>,
    pub targets: Vec<HintTarget<Match>>,
    pub input: String,
    pub message: Option<String>,
}

impl App {
    pub fn from_text(text: &str, matcher: &Matcher) -> Self {
        let lines = split_visible_text(text);
        let targets = matcher
            .find(&lines)
            .into_iter()
            .filter(|hit| hit.text.chars().count() >= 1)
            .collect::<Vec<_>>();
        Self {
            lines,
            targets: assign_hints(targets),
            input: String::new(),
            message: None,
        }
    }

    pub fn handle_char(&mut self, ch: char) -> Outcome {
        if ch == '\u{1b}' || ch == '\u{3}' {
            return Outcome::Cancel;
        }
        if ch == '\u{8}' || ch == '\u{7f}' {
            self.input.pop();
            self.message = None;
            return Outcome::Continue;
        }
        if !ch.is_ascii_alphabetic() {
            return Outcome::Continue;
        }

        self.input.push(ch.to_ascii_lowercase());
        let exact = self.targets.iter().find(|target| target.hint == self.input);
        let has_longer = self
            .targets
            .iter()
            .any(|target| target.hint.starts_with(&self.input) && target.hint != self.input);

        if let Some(target) = exact {
            if !has_longer {
                return Outcome::Copy(target.target.text.clone());
            }
        }

        if self
            .targets
            .iter()
            .any(|target| target.hint.starts_with(&self.input))
        {
            self.message = None;
        } else {
            self.message = Some(format!("no hint starts with {}", self.input));
            self.input.clear();
        }
        Outcome::Continue
    }

    pub fn visible_target_count(&self) -> usize {
        if self.input.is_empty() {
            return self.targets.len();
        }
        self.targets
            .iter()
            .filter(|target| target.hint.starts_with(&self.input))
            .count()
    }
}

fn split_visible_text(text: &str) -> Vec<String> {
    let mut lines = text.lines().map(ToString::to_string).collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_exact_hint() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("open https://example.com", &matcher);
        assert_eq!(app.targets.len(), 1);
        let hint = app.targets[0].hint.clone();
        let mut outcome = Outcome::Continue;
        for ch in hint.chars() {
            outcome = app.handle_char(ch);
        }
        assert_eq!(outcome, Outcome::Copy("https://example.com".to_string()));
    }

    #[test]
    fn backspace_removes_pending_hint_input() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("1234 5678", &matcher);
        app.targets[0].hint = "as".to_string();
        app.targets[1].hint = "ad".to_string();
        assert_eq!(app.handle_char('a'), Outcome::Continue);
        assert_eq!(app.input, "a");
        assert_eq!(app.handle_char('\u{7f}'), Outcome::Continue);
        assert_eq!(app.input, "");
    }

    #[test]
    fn unknown_hint_resets_input() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("1234", &matcher);
        assert_eq!(app.handle_char('z'), Outcome::Continue);
        assert_eq!(app.input, "");
        assert_eq!(app.message.as_deref(), Some("no hint starts with z"));
    }

    #[test]
    fn copies_wrapped_url_without_the_pane_line_break() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("open https://exa\nmple.com", &matcher);
        assert_eq!(app.targets.len(), 1);
        let hint = app.targets[0].hint.clone();
        let mut outcome = Outcome::Continue;
        for ch in hint.chars() {
            outcome = app.handle_char(ch);
        }

        assert_eq!(outcome, Outcome::Copy("https://example.com".to_string()));
    }
}
