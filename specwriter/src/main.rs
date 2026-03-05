use crossterm::{
    event::{Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
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
    let mut events = EventStream::new();
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(100));

    loop {
        terminal.draw(|f| specwriter::ui::draw(f, &app))?;

        // Wait for either a key event, an integrator message, or an animation tick
        tokio::select! {
            event = events.next() => {
                if let Some(Ok(Event::Key(key))) = event {
                    app.handle_key(key);
                }
            }
            msg = ui_rx.recv() => {
                if let Some(msg) = msg {
                    app.update_from_integrator(msg);
                }
            }
            _ = tick_interval.tick() => {
                app.tick();
            }
        }

        // Drain any remaining messages that arrived during the select
        while let Ok(msg) = ui_rx.try_recv() {
            app.update_from_integrator(msg);
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
