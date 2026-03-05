use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use specwriter::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (mut app, mut ui_rx) = App::with_default_integrator();

    loop {
        terminal.draw(|f| specwriter::ui::draw(f, &app))?;

        while let Ok(msg) = ui_rx.try_recv() {
            app.update_from_integrator(msg);
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        app.should_quit = true;
                    }
                    KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        app.submit();
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        app.insert_newline();
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        app.backspace();
                    }
                    KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } => {
                        app.delete();
                    }
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        app.move_left();
                    }
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        app.move_right();
                    }
                    KeyEvent {
                        code: KeyCode::Home,
                        ..
                    } => {
                        app.move_home();
                    }
                    KeyEvent {
                        code: KeyCode::End,
                        ..
                    } => {
                        app.move_end();
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                        ..
                    } => {
                        app.insert_char(c);
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
