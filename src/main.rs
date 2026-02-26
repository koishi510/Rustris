mod audio;
mod game;
mod net;
mod render;
mod ui;

use crossterm::{cursor, execute, terminal};
use std::io;

use game::records::Records;
use game::settings::Settings;

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

    let result: io::Result<()> = (|| {
        loop {
            let mode = match ui::select_mode(&mut stdout, &mut music, &mut settings, &mut records)? {
                Some(m) => m,
                None => return Ok(()),
            };
            if mode == game::GameMode::Versus {
                let action = ui::run_versus_menu(&mut stdout, &mut music, &mut settings)?;
                match action {
                    ui::VersusAction::Host(port) => {
                        let lobby = ui::versus::run_host_lobby(&mut stdout, &mut music, &settings, port)?;
                        if let Some((mut conn, vs_settings)) = lobby {
                            let quit = ui::versus::run_versus(&mut stdout, &mut music, &mut conn, &vs_settings, true)?;
                            if quit {
                                return Ok(());
                            }
                        }
                    }
                    ui::VersusAction::Join(addr) => {
                        let lobby = ui::versus::run_client_lobby(&mut stdout, &mut music, &addr)?;
                        if let Some((mut conn, vs_settings)) = lobby {
                            let quit = ui::versus::run_versus(&mut stdout, &mut music, &mut conn, &vs_settings, false)?;
                            if quit {
                                return Ok(());
                            }
                        }
                    }
                    ui::VersusAction::Back => continue,
                }
                continue;
            }
            match ui::run_game(&mut stdout, mode, &mut music, &mut settings, &mut records)? {
                true => return Ok(()),
                false => continue,
            }
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    match &result {
        Err(e) if e.kind() == io::ErrorKind::Interrupted => Ok(()),
        _ => result,
    }
}
