mod app;
mod helpers;
mod types;

use crate::{app::App, helpers::parse_args};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::*,
};
use std::{io, time::Duration};

const DEFAULT_WORD_COUNT: usize = 512;
const DEFAULT_SECONDS: usize = 60;
const POLLING_RATE_MS: u64 = 16;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (count, seconds, source) = parse_args();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(
        source,
        if count > 0 { count } else { DEFAULT_WORD_COUNT },
        if seconds > 0 {
            seconds
        } else {
            DEFAULT_SECONDS
        },
    );

    loop {
        terminal.draw(|frame| app.draw_ui(frame))?;
        terminal.show_cursor()?;

        if event::poll(Duration::from_millis(POLLING_RATE_MS))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    _ => app.handle_key(key),
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
