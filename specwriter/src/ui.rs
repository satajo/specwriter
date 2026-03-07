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
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).padding(Padding::left(1)))
        .wrap(Wrap { trim: false });
    f.render_widget(input, area);

    // Cursor position (border + padding = 2)
    if app.answer_dialog.is_none() {
        let text_before_cursor = &app.input[..app.cursor_pos];
        let lines: Vec<&str> = text_before_cursor.split('\n').collect();
        let cursor_y = lines.len() - 1;
        let cursor_x = lines.last().map(|l| l.len()).unwrap_or(0);
        f.set_cursor_position(Position::new(
            area.x + 2 + cursor_x as u16,
            area.y + 1 + cursor_y as u16,
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

    // Question list
    let items: Vec<ListItem> = app
        .questions
        .iter()
        .enumerate()
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

    // Detail panel for focused question
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
    let detail = Paragraph::new(detail_text)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(Wrap { trim: false });
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

    let title = format!(" Answer Q{}: {} ", dialog.question.id, dialog.question.text);
    let input = Paragraph::new(dialog.input.as_str())
        .block(Block::default().borders(Borders::ALL).title(title).padding(Padding::left(1)))
        .wrap(Wrap { trim: false });
    f.render_widget(input, dialog_area);

    // Cursor in dialog (border + padding = 2)
    let text_before_cursor = &dialog.input[..dialog.cursor_pos];
    let lines: Vec<&str> = text_before_cursor.split('\n').collect();
    let cursor_y = lines.len() - 1;
    let cursor_x = lines.last().map(|l| l.len()).unwrap_or(0);
    f.set_cursor_position(Position::new(
        dialog_area.x + 2 + cursor_x as u16,
        dialog_area.y + 1 + cursor_y as u16,
    ));
}
