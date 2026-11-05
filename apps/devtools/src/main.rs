mod app;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::event::{self, Event};
use std::time::Duration;

fn main() -> Result<()> {
    let mut app = App::create()?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.should_quit {
        app.poll_loading();
        terminal.draw(|frame| ui::render(app, frame))?;
        if !event::poll(Duration::from_millis(250))? {
            app.handle_tick();
            continue;
        }
        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }
    Ok(())
}
