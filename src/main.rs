mod app;
mod models;
mod nvidia;
mod theme;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let result = run(&mut terminal).await;

    // Cleanup terminal
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;

    result
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Initial fetch
    let _ = app.update_gpu_data();

    loop {
        // Update data every second
        if app.should_update() {
            let _ = app.update_gpu_data();
            app.mark_updated();
        }

        app.tick();

        terminal.draw(|frame| {
            ui::render(frame, &app.gpus, app.frame_count);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
