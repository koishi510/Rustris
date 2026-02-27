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
                'versus: loop {
                    let action = ui::run_versus_menu(&mut stdout, &mut music, &mut settings)?;
                    match action {
                        ui::VersusAction::Host(port) => {
                            match ui::versus::run_host_lobby(&mut stdout, &mut music, port)? {
                                ui::versus::LobbyResult::Connected(mut conn, vs_settings) => {
                                    let quit = ui::versus::run_versus(&mut stdout, &mut music, &mut conn, &vs_settings, true)?;
                                    if quit {
                                        return Ok(());
                                    }
                                }
                                ui::versus::LobbyResult::Back => continue 'versus,
                                ui::versus::LobbyResult::Menu => break 'versus,
                            }
                        }
                        ui::VersusAction::Join(addr) => {
                            match ui::versus::run_client_lobby(&mut stdout, &mut music, &addr)? {
                                ui::versus::LobbyResult::Connected(mut conn, vs_settings) => {
                                    let quit = ui::versus::run_versus(&mut stdout, &mut music, &mut conn, &vs_settings, false)?;
                                    if quit {
                                        return Ok(());
                                    }
                                }
                                ui::versus::LobbyResult::Back => continue 'versus,
                                ui::versus::LobbyResult::Menu => break 'versus,
                            }
                        }
                        ui::VersusAction::Back => break 'versus,
                    }
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
