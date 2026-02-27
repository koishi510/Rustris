use std::io;

use crate::audio;
use crate::game::GameMode;
use crate::game::records::Records;
use crate::game::settings::Settings;

use super::menus::{self, VersusAction};
use super::versus::{self, LobbyResult};
use super::session;

fn run_versus_flow(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
) -> io::Result<()> {
    loop {
        match menus::run_versus_menu(stdout, music, settings)? {
            VersusAction::Host(port) => {
                match versus::run_host_lobby(stdout, music, port)? {
                    LobbyResult::Connected(mut conn, vs_settings) => {
                        if versus::run_versus(stdout, music, &mut conn, &vs_settings, true)? {
                            return Ok(());
                        }
                    }
                    LobbyResult::Back => continue,
                    LobbyResult::Menu => return Ok(()),
                }
            }
            VersusAction::Join(addr) => {
                match versus::run_client_lobby(stdout, music, &addr)? {
                    LobbyResult::Connected(mut conn, vs_settings) => {
                        if versus::run_versus(stdout, music, &mut conn, &vs_settings, false)? {
                            return Ok(());
                        }
                    }
                    LobbyResult::Back => continue,
                    LobbyResult::Menu => return Ok(()),
                }
            }
            VersusAction::Back => return Ok(()),
        }
    }
}

pub fn run_app(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut music = audio::MusicPlayer::new();
    let mut settings = Settings::default();
    let mut records = Records::load();

    loop {
        let mode = match menus::select_mode(stdout, &mut music, &mut settings, &mut records)? {
            Some(m) => m,
            None => return Ok(()),
        };
        if mode == GameMode::Versus {
            run_versus_flow(stdout, &mut music, &mut settings)?;
            continue;
        }
        if session::run_game(stdout, mode, &mut music, &mut settings, &mut records)? {
            return Ok(());
        }
    }
}
