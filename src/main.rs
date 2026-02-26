mod audio;
mod game;
mod net;
mod render;
mod ui;

use crossterm::{cursor, execute, terminal};
use std::io;

use game::records::Records;
use game::settings::Settings;

fn parse_args() -> Option<VersusArg> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                let port = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(3000);
                return Some(VersusArg::Host(port));
            }
            "--join" => {
                let addr = args.get(i + 1).cloned().unwrap_or_else(|| "127.0.0.1:3000".to_string());
                return Some(VersusArg::Join(addr));
            }
            _ => {}
        }
        i += 1;
    }
    None
}

enum VersusArg {
    Host(u16),
    Join(String),
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let mut music = audio::MusicPlayer::new();
    let mut settings = Settings::default();
    let mut records = Records::load();

    let versus_arg = parse_args();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let result: io::Result<()> = (|| {
        // If CLI args specify versus mode, go straight to it
        if let Some(arg) = versus_arg {
            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
            match arg {
                VersusArg::Host(port) => {
                    let lobby = ui::versus::run_host_lobby(&mut stdout, &mut music, &settings, port)?;
                    if let Some((mut conn, vs_settings)) = lobby {
                        return match ui::versus::run_versus(&mut stdout, &mut music, &mut conn, &vs_settings, true)? {
                            true => Ok(()),
                            false => Ok(()),
                        };
                    }
                }
                VersusArg::Join(addr) => {
                    let lobby = ui::versus::run_client_lobby(&mut stdout, &mut music, &addr)?;
                    if let Some((mut conn, vs_settings)) = lobby {
                        return match ui::versus::run_versus(&mut stdout, &mut music, &mut conn, &vs_settings, false)? {
                            true => Ok(()),
                            false => Ok(()),
                        };
                    }
                }
            }
            return Ok(());
        }

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
