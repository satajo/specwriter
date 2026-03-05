use ratatui::{prelude::*, widgets::*};

use crate::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // status bar
            Constraint::Min(5),    // questions
            Constraint::Min(8),    // input
            Constraint::Length(1), // help line
        ])
        .split(f.area());

    // Status bar
    let status = Paragraph::new(app.status.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(status, chunks[0]);

    // Questions
    let q_items: Vec<Line> = if app.questions.is_empty() {
        vec![Line::from("  No open questions yet. Start writing to generate questions.").gray()]
    } else {
        app.questions
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, q)| {
                Line::from(format!("  {}. {}", i + 1, q)).yellow()
            })
            .collect()
    };
    let questions = Paragraph::new(q_items)
        .block(Block::default().borders(Borders::ALL).title(" Open Questions "))
        .wrap(Wrap { trim: false });
    f.render_widget(questions, chunks[1]);

    // Input area
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Input (Ctrl+S to submit) "))
        .wrap(Wrap { trim: false });
    f.render_widget(input, chunks[2]);

    // Calculate cursor position within the input area
    let text_before_cursor = &app.input[..app.cursor_pos];
    let lines: Vec<&str> = text_before_cursor.split('\n').collect();
    let cursor_y = lines.len() - 1;
    let cursor_x = lines.last().map(|l| l.len()).unwrap_or(0);

    // +1 for border offset
    f.set_cursor_position(Position::new(
        chunks[2].x + 1 + cursor_x as u16,
        chunks[2].y + 1 + cursor_y as u16,
    ));

    // Help line
    let help = Paragraph::new(" Ctrl+C: quit | Ctrl+S: submit | Enter: newline")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}
