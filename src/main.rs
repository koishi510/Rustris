mod audio;
mod game;
mod piece;
mod records;
mod render;
mod settings;
mod ui;

use crossterm::{cursor, execute, terminal};
use std::io;

use records::Records;
use settings::Settings;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let mut music = audio::MusicPlayer::new();
    let mut settings = Settings::default();
    let mut records = Records::load();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let result = (|| {
        loop {
            let mode = match ui::select_mode(&mut stdout, &mut music, &mut settings, &mut records)? {
                Some(m) => m,
                None => return Ok(()),
            };
            match ui::run_game(&mut stdout, mode, &mut music, &mut settings, &mut records)? {
                true => return Ok(()),
                false => continue,
            }
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}
