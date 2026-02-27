use crossterm::event::KeyCode;
use std::io;

use crate::audio::{self, Sfx};
use crate::game::GameMode;
use crate::game::records::Records;
use crate::render;
use crate::game::settings::Settings;
use crate::ui::{menu_nav, play_menu_sfx, read_key};

use super::run_settings;

pub fn select_mode(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
    records: &mut Records,
) -> io::Result<Option<GameMode>> {
    let mut mode = GameMode::Marathon;
    let mut sel: usize = 0;
    let count: usize = 6;

    loop {
        render::draw_mode_select(stdout, mode, sel)?;

        if let Some(code) = read_key()? {
            match code {
                KeyCode::Up | KeyCode::Down => {
                    sel = menu_nav(sel, count, code);
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Left => {
                    mode = mode.prev();
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Right => {
                    mode = mode.next();
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Enter => {
                    if sel == 1 {
                        play_menu_sfx(music, Sfx::MenuSelect);
                        return Ok(Some(mode));
                    } else if sel == 2 {
                        play_menu_sfx(music, Sfx::MenuSelect);
                        run_settings(stdout, music, settings, mode, false)?;
                    } else if sel == 3 {
                        play_menu_sfx(music, Sfx::MenuSelect);
                        run_records(stdout, music, records, mode)?;
                    } else if sel == 4 {
                        play_menu_sfx(music, Sfx::MenuSelect);
                        render::draw_help(stdout, 0)?;
                        loop {
                            if let Some(code) = read_key()? {
                                if code == KeyCode::Enter || code == KeyCode::Esc {
                                    play_menu_sfx(music, Sfx::MenuBack);
                                    break;
                                }
                            }
                        }
                    } else if sel == 5 {
                        return Ok(None);
                    }
                }
                _ => {}
            }
        }
    }
}

fn run_records(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    records: &Records,
    initial_mode: GameMode,
) -> io::Result<()> {
    let mut mode = initial_mode;
    let mut sel: usize = 1;

    loop {
        render::draw_records(stdout, records, mode, sel)?;
        if let Some(code) = read_key()? {
            match code {
                KeyCode::Up => {
                    sel = sel.checked_sub(1).unwrap_or(1);
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Down => {
                    sel = 1;
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Left => {
                    if sel == 0 {
                        mode = mode.prev();
                        play_menu_sfx(music, Sfx::MenuMove);
                    }
                }
                KeyCode::Right => {
                    if sel == 0 {
                        mode = mode.next();
                        play_menu_sfx(music, Sfx::MenuMove);
                    }
                }
                KeyCode::Enter => {
                    if sel == 1 {
                        play_menu_sfx(music, Sfx::MenuBack);
                        return Ok(());
                    }
                }
                KeyCode::Esc => {
                    play_menu_sfx(music, Sfx::MenuBack);
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
