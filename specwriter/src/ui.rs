use ratatui::{prelude::*, widgets::*};

use crate::{ActiveTab, App, AppState};

const SPINNER_FRAMES: &[&str] = &[
    ".  ",
    ".. ",
    "...",
    " ..",
    "  .",
    "   ",
];
const SPINNER_TICKS_PER_FRAME: u64 = 1;

fn status_line(app: &App) -> Line<'_> {
    match app.state {
        AppState::Integrating => {
            let frame = ((app.tick / SPINNER_TICKS_PER_FRAME) as usize) % SPINNER_FRAMES.len();
            Line::from(vec![
                Span::styled(SPINNER_FRAMES[frame], Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::raw(&app.status),
            ])
        }
        AppState::Idle => {
            Line::from(Span::raw(&app.status))
        }
        AppState::Error => {
            Line::from(Span::styled(&app.status, Style::default().fg(Color::Red)))
        }
    }
}

/// Calculate center-focused scroll offset.
fn center_scroll(focus: usize, visible: usize, total: usize) -> usize {
    if total <= visible {
        return 0;
    }
    let half = visible / 2;
    let max_scroll = total.saturating_sub(visible);
    focus.saturating_sub(half).min(max_scroll)
}

/// Calculate the visual cursor row and column for text with soft wrapping.
fn wrapped_cursor_pos(text: &str, cursor_pos: usize, inner_width: u16) -> (u16, u16) {
    let w = inner_width.max(1) as usize;
    let before_cursor = &text[..cursor_pos];
    let mut visual_row: usize = 0;

    for (i, line) in before_cursor.split('\n').enumerate() {
        if i > 0 {
            visual_row += 1;
        }
        let len = line.len();
        if len > 0 {
            visual_row += len / w;
        }
    }

    let last_line = before_cursor.rsplit('\n').next().unwrap_or(before_cursor);
    let visual_col = last_line.len() % w;

    (visual_row as u16, visual_col as u16)
}

/// Count total visual lines for wrapped text.
fn wrapped_line_count(text: &str, inner_width: u16) -> usize {
    let w = inner_width.max(1) as usize;
    let mut count: usize = 0;
    for line in text.split('\n') {
        count += if line.is_empty() {
            1
        } else {
            (line.len() + w - 1) / w
        };
    }
    count
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // status
            Constraint::Length(1),  // spacing
            Constraint::Length(1),  // tab labels (no border)
            Constraint::Min(5),    // content (bordered)
            Constraint::Length(1),  // help line
        ])
        .split(f.area());

    // Status line
    f.render_widget(Paragraph::new(status_line(app)), chunks[0]);

    // Tab labels — no block, no borders
    let q_count = app.questions.len();
    let selected = match app.active_tab {
        ActiveTab::TextInput => 0,
        ActiveTab::Questions => 1,
    };
    let highlight = match app.active_tab {
        ActiveTab::TextInput => Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
        ActiveTab::Questions => Style::default().fg(Color::Black).bg(Color::Blue).add_modifier(Modifier::BOLD),
    };
    let titles = vec![
        Line::from(" Text Input ").green(),
        Line::from(format!("Open Questions ({}) ", q_count)).blue(),
    ];
    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(highlight)
        .padding(" ", "")
        .divider("");
    f.render_widget(tabs, chunks[2]);

    // Content area with bordered block
    match app.active_tab {
        ActiveTab::TextInput => draw_text_input(f, app, chunks[3]),
        ActiveTab::Questions => draw_questions(f, app, chunks[3]),
    }

    // Help line
    let help_text = if app.answer_dialog.is_some() {
        " Ctrl+S: submit | Esc: cancel | Enter: newline"
    } else {
        match app.active_tab {
            ActiveTab::TextInput => {
                " Ctrl+C: quit | Tab: switch tab | Ctrl+S: submit | Enter: newline"
            }
            ActiveTab::Questions => {
                " Ctrl+C: quit | Tab: switch tab | \u{2191}\u{2193}: navigate | Enter: answer"
            }
        }
    };
    let help = Paragraph::new(help_text);
    f.render_widget(help, chunks[4]);

    // Answer dialog overlay
    if let Some(ref dialog) = app.answer_dialog {
        draw_answer_dialog(f, dialog, f.area());
    }
}

fn draw_text_input(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::reset())
        .padding(Padding::left(1));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let inner_width = inner.width;

    if app.input.is_empty() {
        let placeholder =
            Paragraph::new("Type your requirements here. Ctrl+S to submit.")
                .style(Style::default().fg(Color::Gray));
        f.render_widget(placeholder, inner);
    } else {
        let (cursor_row, cursor_col) =
            wrapped_cursor_pos(&app.input, app.cursor_pos, inner_width);
        let total_lines = wrapped_line_count(&app.input, inner_width);
        let scroll =
            center_scroll(cursor_row as usize, inner.height as usize, total_lines) as u16;

        let input = Paragraph::new(app.input.as_str())
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));
        f.render_widget(input, inner);

        if app.answer_dialog.is_none() {
            f.set_cursor_position(Position::new(
                inner.x + cursor_col,
                inner.y + cursor_row - scroll,
            ));
        }
    }
}

fn draw_questions(f: &mut Frame, app: &App, area: Rect) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::reset());
    let outer_inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    if app.questions.is_empty() {
        let content = Paragraph::new("  No open questions")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(content, outer_inner);
        return;
    }

    // Split into list and detail
    let sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Min(4)])
        .split(outer_inner);

    // Question list with center-focused scrolling
    let list_inner_height = sub[0].height as usize;
    let list_scroll = center_scroll(app.question_focus, list_inner_height, app.questions.len());

    let items: Vec<ListItem> = app
        .questions
        .iter()
        .enumerate()
        .skip(list_scroll)
        .take(list_inner_height)
        .map(|(i, q)| {
            let line = format!("  Q{} (p{}). {} ({})", q.id, q.priority, q.text, q.file);
            let style = if i == app.question_focus {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();
    let list = List::new(items);
    f.render_widget(list, sub[0]);

    // Detail panel
    let focused = &app.questions[app.question_focus];
    let detail_text = if focused.body.is_empty() {
        format!(
            "Q{} (p{}): {}\n\nFrom: {}",
            focused.id, focused.priority, focused.text, focused.file
        )
    } else {
        format!(
            "Q{} (p{}): {}\n\n{}\n\nFrom: {}",
            focused.id, focused.priority, focused.text, focused.body, focused.file
        )
    };
    let detail_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::reset())
        .title(" Details ");
    let detail_inner = detail_block.inner(sub[1]);
    let detail_inner_height = detail_inner.height as usize;
    let detail_inner_width = detail_inner.width;
    let detail_total = wrapped_line_count(&detail_text, detail_inner_width);
    let detail_scroll = center_scroll(0, detail_inner_height, detail_total) as u16;

    let detail = Paragraph::new(detail_text)
        .block(detail_block)
        .wrap(Wrap { trim: false })
        .scroll((detail_scroll, 0));
    f.render_widget(detail, sub[1]);
}

fn draw_answer_dialog(f: &mut Frame, dialog: &crate::AnswerDialog, area: Rect) {
    let dialog_width = area.width.saturating_sub(10).min(70);
    let dialog_height = 10u16.min(area.height.saturating_sub(6));
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let inner_width = dialog_area.width.saturating_sub(3);
    let inner_height = dialog_area.height.saturating_sub(2);

    let (cursor_row, cursor_col) =
        wrapped_cursor_pos(&dialog.input, dialog.cursor_pos, inner_width);
    let total_lines = wrapped_line_count(&dialog.input, inner_width);
    let scroll = center_scroll(cursor_row as usize, inner_height as usize, total_lines) as u16;

    let title = format!(" Answer Q{}: {} ", dialog.question.id, dialog.question.text);
    let input = Paragraph::new(dialog.input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .padding(Padding::left(1)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(input, dialog_area);

    f.set_cursor_position(Position::new(
        dialog_area.x + 2 + cursor_col,
        dialog_area.y + 1 + cursor_row - scroll,
    ));
}
