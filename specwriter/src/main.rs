use crossterm::{
    event::{Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::io;

use specwriter::App;
use specwriter::integrator::IntegratorConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = IntegratorConfig::default();

    // Parse --specs-dir argument
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--specs-dir" {
            if i + 1 < args.len() {
                config.spec_dir_name = args[i + 1].clone();
                i += 2;
            } else {
                eprintln!("Error: --specs-dir requires a value");
                std::process::exit(1);
            }
        } else {
            i += 1;
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (mut app, mut ui_rx) = App::with_config(config);
    let mut events = EventStream::new();
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(150));

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
