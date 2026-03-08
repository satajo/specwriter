use ratatui::{prelude::*, widgets::*};

use crate::{ActiveTab, AnswerMode, App, AppState, settings::Settings};

const DOT_TICKS_PER_FRAME: u64 = 1;

fn status_line(app: &App) -> Line<'_> {
    match app.state {
        AppState::Integrating => {
            let dot_count = ((app.tick / DOT_TICKS_PER_FRAME) as usize % 3) + 1;
            let dots: String = ".".repeat(dot_count);
            let text = format!(" {}{}", app.status, dots);
            Line::from(Span::styled(text, Style::default().fg(Color::Yellow)))
        }
        AppState::Idle => {
            Line::from(format!(" {}", app.status))
        }
        AppState::Error => {
            Line::from(Span::styled(format!(" {}", app.status), Style::default().fg(Color::Red)))
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

/// Return the style for a priority indicator based on its level.
fn priority_style_for(priority: u8) -> Style {
    match priority {
        5 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        4 => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // status (bordered box)
            Constraint::Length(1),  // tab labels (no border)
            Constraint::Min(5),    // content (bordered)
            Constraint::Length(1),  // help line
        ])
        .split(f.area());

    // Status line in bordered box
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::reset());
    let status = Paragraph::new(status_line(app)).block(status_block);
    f.render_widget(status, chunks[0]);

    // Tab labels — no block, no borders
    let q_count = app.questions.len();
    let selected = match app.active_tab {
        ActiveTab::Writer => 0,
        ActiveTab::Questions => 1,
        ActiveTab::Spec => 2,
        ActiveTab::Settings => 3,
    };
    let highlight = match app.active_tab {
        ActiveTab::Writer => Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
        ActiveTab::Questions => Style::default().fg(Color::Black).bg(Color::Blue).add_modifier(Modifier::BOLD),
        ActiveTab::Spec => Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD),
        ActiveTab::Settings => Style::default().fg(Color::Black).bg(Color::Gray).add_modifier(Modifier::BOLD),
    };
    let spec_line_count = app.spec_content.as_ref().map(|c| c.lines().count()).unwrap_or(0);
    let spec_label = format!(" {} ({}) ", app.integrator.spec_filename(), spec_line_count);
    let titles = vec![
        Line::from(" Writer ").green(),
        Line::from(format!(" Open questions ({}) ", q_count)).blue(),
        Line::from(spec_label).red(),
        Line::from(" Settings ").gray(),
    ];
    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(highlight)
        .padding("", "")
        .divider("");
    f.render_widget(tabs, chunks[1]);

    // Content area with bordered block
    match app.active_tab {
        ActiveTab::Writer => draw_text_input(f, app, chunks[2]),
        ActiveTab::Questions => draw_questions(f, app, chunks[2]),
        ActiveTab::Spec => draw_spec(f, app, chunks[2]),
        ActiveTab::Settings => draw_settings(f, app, chunks[2]),
    }

    // Help line
    let help_text = if app.quit_dialog {
        " Ctrl+C: confirm quit | Esc: cancel"
    } else if let Some(ref dialog) = app.answer_dialog {
        match dialog.mode {
            AnswerMode::SelectSolution { .. } => {
                " Enter: select | \u{2191}\u{2193}: navigate | Esc: cancel"
            }
            AnswerMode::WriteCustom => {
                if dialog.question.solutions.is_empty() {
                    " Ctrl+S: submit | Esc: cancel | Enter: newline"
                } else {
                    " Ctrl+S: submit | Esc: back | Enter: newline"
                }
            }
        }
    } else {
        match app.active_tab {
            ActiveTab::Writer => {
                " Ctrl+C: quit | Tab: switch tab | Ctrl+S: submit | Enter: newline"
            }
            ActiveTab::Questions => {
                " Ctrl+C: quit | Tab: switch tab | \u{2191}\u{2193}: navigate | Enter: answer"
            }
            ActiveTab::Spec => {
                " Ctrl+C: quit | Tab: switch tab | \u{2191}\u{2193}: scroll"
            }
            ActiveTab::Settings => {
                if app.settings_save_dialog {
                    " Enter: save | Esc: cancel"
                } else if app.settings_editing.is_some() {
                    " Enter: confirm | Esc: cancel | type to edit"
                } else {
                    " Ctrl+C: quit | Tab: switch tab | \u{2191}\u{2193}: navigate | Enter: edit/toggle | Ctrl+S: save"
                }
            }
        }
    };
    let help = Paragraph::new(help_text);
    f.render_widget(help, chunks[3]);

    // Dialog overlays
    if app.quit_dialog {
        draw_quit_dialog(f, f.area());
    } else if app.settings_save_dialog {
        draw_settings_save_dialog(f, f.area());
    } else if let Some(ref dialog) = app.answer_dialog {
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
        let placeholder_text = if app.state == AppState::Integrating {
            "Type your requirements here. Ctrl+S to add to queue."
        } else {
            "Type your requirements here. Ctrl+S to submit."
        };
        let placeholder =
            Paragraph::new(placeholder_text)
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
    if app.questions.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::reset());
        let inner = block.inner(area);
        f.render_widget(block, area);
        let content = Paragraph::new("  No open questions")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(content, inner);
        return;
    }

    // Split 50/50 into list and detail — each with its own bordered box
    let sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Question list box
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::reset());
    let list_inner = list_block.inner(sub[0]);
    f.render_widget(list_block, sub[0]);

    let list_inner_height = list_inner.height as usize;
    let list_scroll = center_scroll(app.question_focus, list_inner_height, app.questions.len());

    let items: Vec<ListItem> = app
        .questions
        .iter()
        .enumerate()
        .skip(list_scroll)
        .take(list_inner_height)
        .map(|(i, q)| {
            let display_priority = q.priority.min(5);
            let priority_text = format!("  [{}] ", display_priority);
            let priority_style = if i == app.question_focus {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                priority_style_for(display_priority)
            };
            let title_style = if i == app.question_focus {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };
            let line = Line::from(vec![
                Span::styled(priority_text, priority_style),
                Span::styled(q.text.clone(), title_style),
            ]);
            ListItem::new(line)
        })
        .collect();
    let list = List::new(items);
    f.render_widget(list, list_inner);

    // Detail box
    let focused = &app.questions[app.question_focus];
    let display_priority = focused.priority.min(5);
    let detail_text = if focused.body.is_empty() {
        format!("[{}] {}", display_priority, focused.text)
    } else {
        format!("[{}] {}\n\n{}", display_priority, focused.text, focused.body)
    };
    let detail_block = Block::default()
        .borders(Borders::ALL)
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

fn draw_spec(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::reset())
        .padding(Padding::left(1));

    match &app.spec_content {
        None => {
            let inner = block.inner(area);
            f.render_widget(block, area);
            let placeholder = Paragraph::new("No spec file yet — submit requirements to create one.")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(placeholder, inner);
        }
        Some(content) => {
            let paragraph = Paragraph::new(content.as_str())
                .block(block)
                .scroll((app.spec_scroll, 0));
            f.render_widget(paragraph, area);
        }
    }
}

fn draw_settings(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::reset())
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let label_width = 18;

    let mut items: Vec<ListItem> = Vec::new();
    for i in 0..Settings::COUNT {
        let label = Settings::label(i);
        let is_focused = i == app.settings_focus;
        let is_editing = is_focused && app.settings_editing.is_some();

        let value_str = if is_editing {
            let edit = app.settings_editing.as_ref().unwrap();
            if edit.buffer.is_empty() {
                " ".to_string()
            } else {
                edit.buffer.clone()
            }
        } else {
            app.settings.display_value(i)
        };

        let padded_label = format!("{:width$}", label, width = label_width);

        let style = if is_focused {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default()
        };

        let value_style = if !is_focused && app.settings.display_value(i) == "(not set)" {
            Style::default().fg(Color::Gray)
        } else if is_focused {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(padded_label, style),
            Span::styled(value_str, value_style),
        ]);
        items.push(ListItem::new(line));
    }
    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_quit_dialog(f: &mut Frame, area: Rect) {
    let dialog_width = 60u16.min(area.width.saturating_sub(10));
    let dialog_height = 5u16.min(area.height.saturating_sub(6));
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let body = Paragraph::new("Integration in progress. Press Ctrl+C again to quit.")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm Quit ")
                .padding(Padding::new(1, 0, 1, 0)),
        );
    f.render_widget(body, dialog_area);
}

fn draw_settings_save_dialog(f: &mut Frame, area: Rect) {
    let dialog_width = 60u16.min(area.width.saturating_sub(10));
    let dialog_height = 5u16.min(area.height.saturating_sub(6));
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let body = Paragraph::new("Save settings? Restart required for changes to take effect.")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Save Settings ")
                .padding(Padding::new(1, 0, 1, 0)),
        );
    f.render_widget(body, dialog_area);
}

fn draw_answer_dialog(f: &mut Frame, dialog: &crate::AnswerDialog, area: Rect) {
    let dialog_width = area.width.saturating_sub(10).min(70);
    let is_solution_mode = matches!(dialog.mode, AnswerMode::SelectSolution { .. });
    let max_height = if is_solution_mode { 20u16 } else { 10u16 };
    let dialog_height = max_height.min(area.height.saturating_sub(6));
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let title = format!(" Answer: {} ", dialog.question.text);

    match dialog.mode {
        AnswerMode::SelectSolution { focus } => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .padding(Padding::left(1));
            let inner = block.inner(dialog_area);
            f.render_widget(block, dialog_area);

            // Split inner area 40/60 for list/detail
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(inner);

            let num_solutions = dialog.question.solutions.len();
            let mut items: Vec<ListItem> = dialog.question.solutions.iter().enumerate().map(|(i, sol)| {
                let style = if i == focus {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(format!("  {}", sol.title)).style(style)
            }).collect();

            // "Write custom answer..." as last item
            let custom_style = if focus == num_solutions {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
            };
            items.push(ListItem::new("  Write custom answer...").style(custom_style));

            let list = List::new(items);
            f.render_widget(list, chunks[0]);

            // Detail pane showing focused solution body
            let detail_block = Block::default()
                .borders(Borders::ALL)
                .title(" Details ")
                .padding(Padding::horizontal(1));
            let detail_inner = detail_block.inner(chunks[1]);

            if focus < num_solutions {
                let body = &dialog.question.solutions[focus].body;
                let total_lines = wrapped_line_count(body, detail_inner.width);
                let scroll = center_scroll(0, detail_inner.height as usize, total_lines) as u16;
                let detail = Paragraph::new(body.as_str())
                    .wrap(Wrap { trim: false })
                    .scroll((scroll, 0))
                    .block(detail_block);
                f.render_widget(detail, chunks[1]);
            } else {
                f.render_widget(detail_block, chunks[1]);
            }
        }
        AnswerMode::WriteCustom => {
            let inner_width = dialog_area.width.saturating_sub(3);
            let inner_height = dialog_area.height.saturating_sub(2);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .padding(Padding::left(1));

            if dialog.input.is_empty() {
                let placeholder = Paragraph::new("Type your answer here. Ctrl+S to submit.")
                    .style(Style::default().fg(Color::Gray))
                    .block(block);
                f.render_widget(placeholder, dialog_area);
            } else {
                let (cursor_row, cursor_col) =
                    wrapped_cursor_pos(&dialog.input, dialog.cursor_pos, inner_width);
                let total_lines = wrapped_line_count(&dialog.input, inner_width);
                let scroll =
                    center_scroll(cursor_row as usize, inner_height as usize, total_lines) as u16;

                let input = Paragraph::new(dialog.input.as_str())
                    .block(block)
                    .wrap(Wrap { trim: false })
                    .scroll((scroll, 0));
                f.render_widget(input, dialog_area);

                f.set_cursor_position(Position::new(
                    dialog_area.x + 2 + cursor_col,
                    dialog_area.y + 1 + cursor_row - scroll,
                ));
            }
        }
    }
}
