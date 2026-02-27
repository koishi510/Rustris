mod audio;
mod game;
mod net;
mod render;
mod ui;

use crossterm::{cursor, execute, terminal};
use std::io;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let result = ui::run_app(&mut stdout);

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    match &result {
        Err(e) if e.kind() == io::ErrorKind::Interrupted => Ok(()),
        _ => result,
    }
}
