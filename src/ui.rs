use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::hints::HintTarget;
use crate::patterns::Match;

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let lines = render_lines(app, usize::from(area.height.saturating_sub(1)));
    frame.render_widget(Paragraph::new(lines), area);
    draw_status(frame, app, area);
}

fn draw_status(frame: &mut Frame<'_>, app: &App, area: Rect) {
    if area.height == 0 {
        return;
    }
    let status_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    let message = app.message.as_deref().unwrap_or("");
    let text = format!(
        " herdr-tiny-fingers  hints: {}  input: {}  {}  esc/ctrl-c: close ",
        app.visible_target_count(),
        if app.input.is_empty() {
            "-"
        } else {
            &app.input
        },
        message
    );
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Black).bg(Color::Yellow)),
        status_area,
    );
}

pub fn render_lines(app: &App, max_lines: usize) -> Vec<Line<'static>> {
    if app.targets.is_empty() {
        return vec![Line::from(vec![Span::styled(
            "No tmux-fingers built-in pattern matches on this visible screen.",
            Style::default().fg(Color::Yellow),
        )])];
    }
    app.lines
        .iter()
        .enumerate()
        .take(max_lines)
        .map(|(row, line)| render_line(row, line, &app.targets, &app.input))
        .collect()
}

pub fn render_line(
    row: usize,
    line: &str,
    targets: &[HintTarget<Match>],
    input: &str,
) -> Line<'static> {
    let line_chars = line.chars().collect::<Vec<_>>();
    let mut spans = Vec::new();
    let mut col = 0;
    let mut row_targets = targets
        .iter()
        .filter(|target| target.target.full_start.row == row)
        .filter(|target| input.is_empty() || target.hint.starts_with(input))
        .collect::<Vec<_>>();
    row_targets.sort_by_key(|target| target.target.full_start.col);

    for target in row_targets {
        let full_start = target.target.full_start.col;
        let full_end = target.target.full_end.col.min(line_chars.len());
        let capture_start = target.target.start.col;
        let capture_end = target.target.end.col.min(line_chars.len());
        if full_start < col
            || full_end <= full_start
            || target.hint.chars().count() > target.target.text.chars().count()
        {
            continue;
        }
        if col < full_start {
            spans.push(Span::raw(chars_to_string(&line_chars[col..full_start])));
        }
        if full_start < capture_start {
            spans.push(Span::styled(
                chars_to_string(&line_chars[full_start..capture_start]),
                Style::default().fg(Color::Cyan),
            ));
        }
        let hint_width = target.hint.chars().count();
        spans.push(Span::styled(
            target.hint.clone(),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
        let highlight_start = capture_start.saturating_add(hint_width);
        if highlight_start < capture_end {
            spans.push(Span::styled(
                chars_to_string(&line_chars[highlight_start..capture_end]),
                Style::default().fg(Color::Black).bg(Color::Cyan),
            ));
        }
        if capture_end < full_end {
            spans.push(Span::styled(
                chars_to_string(&line_chars[capture_end..full_end]),
                Style::default().fg(Color::Cyan),
            ));
        }
        col = full_end;
    }
    if col < line_chars.len() {
        spans.push(Span::raw(chars_to_string(&line_chars[col..])));
    }
    Line::from(spans)
}

fn chars_to_string(chars: &[char]) -> String {
    chars.iter().collect()
}

#[allow(dead_code)]
fn _block() -> Block<'static> {
    Block::default()
}

#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::patterns::Matcher;

    use super::*;

    #[test]
    fn render_line_replaces_start_of_match_with_hint_without_changing_text_width() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("open https://example.com now", &matcher);
        app.targets[0].hint = "a".to_string();
        let line = render_line(0, &app.lines[0], &app.targets, "");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert_eq!(rendered, "open attps://example.com now");
        assert_eq!(rendered.chars().count(), app.lines[0].chars().count());
    }

    #[test]
    fn render_filters_to_current_hint_prefix() {
        let matcher = Matcher::builtin().unwrap();
        let mut app = App::from_text("1234 5678", &matcher);
        app.targets[0].hint = "a".to_string();
        app.targets[1].hint = "s".to_string();
        let line = render_line(0, &app.lines[0], &app.targets, "s");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert_eq!(rendered, "1234 s678");
    }
}
