use ratatui::{prelude::*, widgets::*};

use crate::{ActiveTab, App, AppState};

// ◰ ◳ ◲ ◱ — small square rotating through corners
const SPINNER_FRAMES: &[&str] = &["\u{25f0}", "\u{25f3}", "\u{25f2}", "\u{25f1}"];
const SPINNER_TICKS_PER_FRAME: u64 = 1; // advance every 150ms (brisk pace)

fn status_indicator(app: &App) -> (&str, Style) {
    match app.state {
        AppState::Integrating => {
            let frame = ((app.tick / SPINNER_TICKS_PER_FRAME) as usize) % SPINNER_FRAMES.len();
            (SPINNER_FRAMES[frame], Style::default().fg(Color::Yellow))
        }
        AppState::Idle => {
            ("\u{25f3}", Style::default().fg(Color::Green)) // ◳
        }
        AppState::Error => {
            ("\u{25f1}", Style::default().fg(Color::Red)) // ◱
        }
    }
}

/// Calculate center-focused scroll offset.
/// Keeps `focus` roughly centered in a viewport of `visible` height,
/// pinning to top/bottom when near the edges.
fn center_scroll(focus: usize, visible: usize, total: usize) -> usize {
    if total <= visible {
        return 0;
    }
    let half = visible / 2;
    let max_scroll = total.saturating_sub(visible);
    focus.saturating_sub(half).min(max_scroll)
}

/// Calculate the visual cursor row and column for text with soft wrapping.
/// `inner_width` is the number of columns available for text (after borders/padding).
/// Returns (visual_row, visual_col).
fn wrapped_cursor_pos(text: &str, cursor_pos: usize, inner_width: u16) -> (u16, u16) {
    let w = inner_width.max(1) as usize;
    let before_cursor = &text[..cursor_pos];
    let mut visual_row: usize = 0;

    for (i, line) in before_cursor.split('\n').enumerate() {
        if i > 0 {
            visual_row += 1; // newline itself advances a row
        }
        let len = line.len();
        if len > 0 {
            // Lines that fill exact multiples of width don't get an extra wrap line
            visual_row += len / w;
        }
    }

    // The cursor column is the position within the last visual line
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
            (line.len() + w - 1) / w // ceil division
        };
    }
    count
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // status (plain text, no border)
            Constraint::Length(1),  // empty spacing line
            Constraint::Length(1),  // tab bar
            Constraint::Min(10),   // tab content
            Constraint::Length(1), // help line
        ])
        .split(f.area());

    // Status line (plain text, no border)
    let (icon, icon_style) = status_indicator(app);
    let status_line = Line::from(vec![
        Span::styled(format!("{} ", icon), icon_style),
        Span::raw(&app.status),
    ]);
    f.render_widget(Paragraph::new(status_line), chunks[0]);

    // chunks[1] is the empty spacing line — left blank

    // Tab bar
    let q_count = app.questions.len();
    let text_input_style = if app.active_tab == ActiveTab::TextInput {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else {
        Style::default().fg(Color::Green)
    };
    let questions_label = format!(" Open Questions ({}) ", q_count);
    let questions_style = if app.active_tab == ActiveTab::Questions {
        Style::default().fg(Color::Black).bg(Color::Blue)
    } else {
        Style::default().fg(Color::Blue)
    };
    let tab_bar = Line::from(vec![
        Span::styled(" Text Input ", text_input_style),
        Span::raw(" "),
        Span::styled(questions_label, questions_style),
    ]);
    f.render_widget(Paragraph::new(tab_bar), chunks[2]);

    // Tab content
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
    // inner width: area - 2 (borders) - 1 (left padding)
    let inner_width = area.width.saturating_sub(3);
    let inner_height = area.height.saturating_sub(2); // borders top+bottom

    let (cursor_row, cursor_col) = wrapped_cursor_pos(&app.input, app.cursor_pos, inner_width);
    let total_lines = wrapped_line_count(&app.input, inner_width);
    let scroll = center_scroll(cursor_row as usize, inner_height as usize, total_lines) as u16;

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).padding(Padding::left(1)))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(input, area);

    // Cursor position
    if app.answer_dialog.is_none() {
        f.set_cursor_position(Position::new(
            area.x + 2 + cursor_col,
            area.y + 1 + cursor_row - scroll,
        ));
    }
}

fn draw_questions(f: &mut Frame, app: &App, area: Rect) {
    if app.questions.is_empty() {
        let content = Paragraph::new("  No open questions").gray().block(
            Block::default().borders(Borders::ALL),
        );
        f.render_widget(content, area);
        return;
    }

    // Split into list and detail
    let sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Min(4)])
        .split(area);

    // Question list with center-focused scrolling
    let visible_items = sub[0].height.saturating_sub(2) as usize; // minus borders
    let list_scroll = center_scroll(app.question_focus, visible_items, app.questions.len());

    let items: Vec<ListItem> = app
        .questions
        .iter()
        .enumerate()
        .skip(list_scroll)
        .take(visible_items)
        .map(|(i, q)| {
            let line = format!("  Q{} (p{}). {} ({})", q.id, q.priority, q.text, q.file);
            let style = if i == app.question_focus {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::Yellow)
            };
            ListItem::new(line).style(style)
        })
        .collect();
    let list = List::new(items).block(
        Block::default().borders(Borders::ALL),
    );
    f.render_widget(list, sub[0]);

    // Detail panel for focused question with scroll support
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
    let detail_inner_width = sub[1].width.saturating_sub(2);
    let detail_inner_height = sub[1].height.saturating_sub(2) as usize;
    let detail_total = wrapped_line_count(&detail_text, detail_inner_width);
    let detail_scroll = center_scroll(0, detail_inner_height, detail_total) as u16;

    let detail = Paragraph::new(detail_text)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(Wrap { trim: false })
        .scroll((detail_scroll, 0));
    f.render_widget(detail, sub[1]);
}

fn draw_answer_dialog(f: &mut Frame, dialog: &crate::AnswerDialog, area: Rect) {
    // Center the dialog
    let dialog_width = area.width.saturating_sub(10).min(70);
    let dialog_height = 10u16.min(area.height.saturating_sub(6));
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear background
    f.render_widget(Clear, dialog_area);

    // inner width: dialog - 2 (borders) - 1 (left padding)
    let inner_width = dialog_area.width.saturating_sub(3);
    let inner_height = dialog_area.height.saturating_sub(2);

    let (cursor_row, cursor_col) = wrapped_cursor_pos(&dialog.input, dialog.cursor_pos, inner_width);
    let total_lines = wrapped_line_count(&dialog.input, inner_width);
    let scroll = center_scroll(cursor_row as usize, inner_height as usize, total_lines) as u16;

    let title = format!(" Answer Q{}: {} ", dialog.question.id, dialog.question.text);
    let input = Paragraph::new(dialog.input.as_str())
        .block(Block::default().borders(Borders::ALL).title(title).padding(Padding::left(1)))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(input, dialog_area);

    // Cursor in dialog
    f.set_cursor_position(Position::new(
        dialog_area.x + 2 + cursor_col,
        dialog_area.y + 1 + cursor_row - scroll,
    ));
}
