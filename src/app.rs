use crate::hints::{assign_hints, HintTarget};
use crate::patterns::{Match, Matcher};
use crate::theme::Theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Copy(String),
    CopyMultiple(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct App {
    pub lines: Vec<String>,
    pub targets: Vec<HintTarget<Match>>,
    pub input: String,
    pub message: Option<String>,
    pub multi_mode: bool,
    pub selected_hints: Vec<String>,
    pub theme: Theme,
}

impl App {
    pub fn from_text(text: &str, matcher: &Matcher) -> Self {
        Self::from_text_with_theme(text, matcher, Theme::default())
    }

    pub fn from_text_with_theme(text: &str, matcher: &Matcher, theme: Theme) -> Self {
        Self::from_text_with_theme_and_optional_pane_width(text, matcher, theme, None)
    }

    pub fn from_text_with_theme_and_pane_width(
        text: &str,
        matcher: &Matcher,
        theme: Theme,
        pane_width: usize,
    ) -> Self {
        Self::from_text_with_theme_and_optional_pane_width(text, matcher, theme, Some(pane_width))
    }

    fn from_text_with_theme_and_optional_pane_width(
        text: &str,
        matcher: &Matcher,
        theme: Theme,
        pane_width: Option<usize>,
    ) -> Self {
        let lines = split_visible_text(text);
        let hits = pane_width
            .map(|width| matcher.find_with_wrap_width(&lines, width))
            .unwrap_or_else(|| matcher.find(&lines));
        let targets = hits
            .into_iter()
            .filter(|hit| hit.text.chars().count() >= 1)
            .collect::<Vec<_>>();
        Self {
            lines,
            targets: assign_hints(targets),
            input: String::new(),
            message: None,
            multi_mode: false,
            selected_hints: Vec::new(),
            theme,
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
        if ch == '\t' {
            return self.handle_tab();
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
                if self.multi_mode {
                    self.toggle_selected_hint(target.hint.clone());
                    self.input.clear();
                    self.message = None;
                    return Outcome::Continue;
                }
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

    fn handle_tab(&mut self) -> Outcome {
        if !self.multi_mode {
            self.multi_mode = true;
            self.input.clear();
            self.message = Some("multi mode".to_string());
            return Outcome::Continue;
        }

        if self.selected_hints.is_empty() {
            self.message = Some("multi mode: no selections".to_string());
            return Outcome::Continue;
        }

        Outcome::CopyMultiple(self.selected_text().join("\n"))
    }

    fn toggle_selected_hint(&mut self, hint: String) {
        if let Some(index) = self
            .selected_hints
            .iter()
            .position(|selected_hint| selected_hint == &hint)
        {
            self.selected_hints.remove(index);
        } else {
            self.selected_hints.push(hint);
        }
    }

    fn selected_text(&self) -> Vec<String> {
        self.selected_hints
            .iter()
            .filter_map(|hint| {
                self.targets
                    .iter()
                    .find(|target| &target.hint == hint)
                    .map(|target| target.target.text.clone())
            })
            .collect()
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

    pub fn selected_target_count(&self) -> usize {
        self.selected_hints.len()
    }

    pub fn is_selected_hint(&self, hint: &str) -> bool {
        self.selected_hints
            .iter()
            .any(|selected_hint| selected_hint == hint)
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

    #[test]
    fn copies_a_path_wrapped_at_the_actual_pane_width() {
        let matcher = Matcher::builtin().unwrap();
        let first_line =
            "• /home/hotchpotch/src/github.com/hotchpotch/mmBERT-embedding-reranker/static-small-embeddings/scripts/";
        let mut app = App::from_text_with_theme_and_pane_width(
            &format!("{first_line}\n  infer_nq100k_bf16.py --- wrapped path"),
            &matcher,
            Theme::default(),
            first_line.chars().count(),
        );

        assert_eq!(app.targets.len(), 1);
        let hint = app.targets[0].hint.clone();
        let mut outcome = Outcome::Continue;
        for ch in hint.chars() {
            outcome = app.handle_char(ch);
        }

        assert_eq!(
            outcome,
            Outcome::Copy(
                "/home/hotchpotch/src/github.com/hotchpotch/mmBERT-embedding-reranker/static-small-embeddings/scripts/infer_nq100k_bf16.py".to_string()
            )
        );
    }

    #[test]
    fn tab_toggles_multi_mode_and_copies_selected_matches() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("1234 5678", &matcher);
        app.targets[0].hint = "a".to_string();
        app.targets[1].hint = "s".to_string();

        assert_eq!(app.handle_char('\t'), Outcome::Continue);
        assert!(app.multi_mode);
        assert_eq!(app.handle_char('a'), Outcome::Continue);
        assert_eq!(app.handle_char('s'), Outcome::Continue);
        assert_eq!(app.selected_target_count(), 2);

        assert_eq!(
            app.handle_char('\t'),
            Outcome::CopyMultiple("1234\n5678".to_string())
        );
    }

    #[test]
    fn multi_mode_can_toggle_a_selected_match_off() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("1234 5678", &matcher);
        app.targets[0].hint = "a".to_string();

        assert_eq!(app.handle_char('\t'), Outcome::Continue);
        assert_eq!(app.handle_char('a'), Outcome::Continue);
        assert_eq!(app.handle_char('a'), Outcome::Continue);

        assert_eq!(app.selected_target_count(), 0);
    }
}
