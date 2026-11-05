use crate::app::{ActiveMode, App};
use devtools_core::{Block, TextStyle};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Paragraph, Wrap},
};

pub fn render(app: &App, frame: &mut Frame) {
    let candidates = app.get_completions();
    let areas = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(candidates.len().min(5) as u16),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());
    render_title(app, frame, areas[0]);
    render_content(app, frame, areas[1]);
    render_completions(app, frame, areas[2], &candidates);
    render_status(app, frame, areas[3]);
    render_input(app, frame, areas[4]);
}

fn render_title(app: &App, frame: &mut Frame, area: Rect) {
    let title = match &app.mode {
        ActiveMode::Shell => "devtools".to_string(),
        ActiveMode::Extension { name, .. } | ActiveMode::Loading { name, .. } => {
            format!("devtools / {name}")
        }
    };
    frame.render_widget(Paragraph::new(title), area);
}

fn render_content(app: &App, frame: &mut Frame, area: Rect) {
    let muted = Style::default().fg(Color::DarkGray);
    let mut lines = Vec::new();
    match &app.mode {
        ActiveMode::Loading { name, started, .. } => {
            let elapsed = started.elapsed();
            let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let icon = spinner[(elapsed.as_millis() / 100) as usize % spinner.len()];
            lines.push(Line::raw(format!(
                "{icon} Loading {name}… {:.1}s",
                elapsed.as_secs_f64()
            )));
            lines.push(Line::styled("Esc to cancel", muted));
        }
        ActiveMode::Shell => {
            lines.push(Line::styled(
                "/help  ·  ↑↓ select  ·  Enter/Tab choose  ·  Ctrl+C quit",
                muted,
            ));
        }
        ActiveMode::Extension {
            current_view: Some(view),
            ..
        } => {
            for block in &view.blocks {
                render_block(block, &mut lines);
            }
        }
        ActiveMode::Extension {
            current_view: None, ..
        } => {
            lines.push(Line::raw("No view returned by extension."));
        }
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn render_block<'a>(block: &'a Block, lines: &mut Vec<Line<'a>>) {
    let muted = Style::default().fg(Color::DarkGray);
    match block {
        Block::Header(title) => lines.push(Line::raw(title)),
        Block::Text { text, style } => {
            let style = if *style == TextStyle::Error {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            lines.push(Line::styled(text, style));
        }
        Block::KeyValue { items } => {
            for (key, value) in items {
                lines.push(Line::raw(format!("{key}: {value}")));
            }
        }
        Block::Table { headers, rows } => {
            lines.push(Line::raw(headers.join("  ")));
            for row in rows {
                lines.push(Line::raw(row.join("  ")));
            }
        }
        Block::Help(hints) => {
            lines.push(Line::raw(""));
            for (command, description) in hints {
                lines.push(Line::styled(format!("{command}  {description}"), muted));
            }
        }
    }
}

fn render_completions(app: &App, frame: &mut Frame, area: Rect, candidates: &[String]) {
    let muted = Style::default().fg(Color::DarkGray);
    let first = app.completion_index.saturating_sub(3);
    let suggestions: Vec<Line> = candidates
        .iter()
        .enumerate()
        .skip(first)
        .take(5)
        .map(|(index, candidate)| {
            let style = if index == app.completion_index {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                muted
            };
            Line::styled(format!(" {candidate} "), style)
        })
        .collect();
    frame.render_widget(Paragraph::new(suggestions), area);
}

fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    let muted = Style::default().fg(Color::DarkGray);
    let status_style = if app.status_is_error {
        Style::default().fg(Color::Red)
    } else {
        muted
    };
    frame.render_widget(
        Paragraph::new(app.status_message.as_str()).style(status_style),
        area,
    );
}

fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let prompt = match &app.mode {
        ActiveMode::Shell => "> ".to_string(),
        ActiveMode::Extension { name, .. } | ActiveMode::Loading { name, .. } => {
            format!("{name}> ")
        }
    };
    let input = Line::raw(format!("{prompt}{}", app.input));
    let cursor = Line::raw(format!("{prompt}{}", &app.input[..app.input_cursor])).width();
    let scroll = cursor.saturating_sub(area.width.saturating_sub(1) as usize);
    frame.render_widget(Paragraph::new(input).scroll((0, scroll as u16)), area);
    if area.width > 0 && area.height > 0 {
        frame.set_cursor_position((area.x + (cursor - scroll) as u16, area.y));
    }
}
