use crossterm::event::KeyCode;
use std::io;

use crate::audio::{self, Sfx};
use crate::game::GameMode;
use crate::game::piece::MAX_NEXT_COUNT;
use crate::render;
use crate::game::settings::Settings;
use crate::ui::{menu_nav, play_menu_sfx, read_key};

fn adjust_level_cap(settings: &mut Settings, direction: i32) {
    match (settings.level_cap, direction) {
        (Some(c), 1) if c >= 20 => settings.level_cap = None,
        (Some(c), 1) => settings.level_cap = Some((c + 1).min(20)),
        (Some(c), -1) => settings.level_cap = Some(if c <= 1 { 1 } else { c - 1 }),
        (None, -1) => settings.level_cap = Some(20),
        _ => {}
    }
}

fn adjust_setting(settings: &mut Settings, sel: usize, direction: i32, mode: GameMode) {
    let mc: usize = match mode {
        GameMode::Marathon => 3,
        GameMode::Endless => 2,
        GameMode::Sprint | GameMode::Ultra | GameMode::Versus => 1,
    };

    if sel < mc {
        match mode {
            GameMode::Marathon => match sel {
                0 => {
                    let v = settings.level as i32 + direction;
                    settings.level = v.clamp(1, 20) as u32;
                }
                1 => {
                    let v = settings.marathon_goal as i32 + direction * 10;
                    settings.marathon_goal = v.clamp(10, 300) as u32;
                }
                2 => {
                    adjust_level_cap(settings, direction);
                }
                _ => {}
            },
            GameMode::Endless => match sel {
                0 => {
                    let v = settings.level as i32 + direction;
                    settings.level = v.clamp(1, 20) as u32;
                }
                1 => {
                    adjust_level_cap(settings, direction);
                }
                _ => {}
            },
            GameMode::Sprint => {
                let v = settings.sprint_goal as i32 + direction * 10;
                settings.sprint_goal = v.clamp(10, 100) as u32;
            }
            GameMode::Ultra => {
                let v = settings.ultra_time as i32 + direction * 10;
                settings.ultra_time = v.clamp(30, 300) as u32;
            }
            GameMode::Versus => {
                let v = settings.level as i32 + direction;
                settings.level = v.clamp(1, 20) as u32;
            }
        }
    } else if sel == mc {
        let v = settings.next_count as i32 + direction;
        settings.next_count = v.clamp(0, MAX_NEXT_COUNT as i32) as usize;
    } else if sel == mc + 1 {
        let v = settings.lock_delay_ms as i32 + direction * 100;
        settings.lock_delay_ms = v.clamp(0, 2000) as u32;
    } else if sel == mc + 2 {
        match (settings.move_reset, direction) {
            (Some(n), 1) if n >= 30 => settings.move_reset = None,
            (Some(n), 1) => settings.move_reset = Some((n + 1).min(30)),
            (Some(0), -1) => {}
            (Some(n), -1) => settings.move_reset = Some(n - 1),
            (None, -1) => settings.move_reset = Some(30),
            _ => {}
        }
    } else if sel == mc + 3 {
        settings.ghost = !settings.ghost;
    } else if sel == mc + 4 {
        settings.line_clear_anim = !settings.line_clear_anim;
    } else if sel == mc + 5 {
        settings.bag_randomizer = !settings.bag_randomizer;
    } else if sel == mc + 6 {
        settings.srs_enabled = !settings.srs_enabled;
    } else if sel == mc + 7 {
        settings.hold_enabled = !settings.hold_enabled;
    }
}

pub(crate) fn run_settings(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
    mode: GameMode,
    in_game: bool,
) -> io::Result<()> {
    let mut sel: usize = 0;

    if in_game {
        let count: usize = 3;
        loop {
            let (bgm_on, sfx_on) = match music.as_ref() {
                Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
                None => (false, false),
            };
            render::draw_settings(stdout, settings, mode, bgm_on, sfx_on, sel, true)?;
            if let Some(code) = read_key()? {
                match code {
                    KeyCode::Up | KeyCode::Down => {
                        sel = menu_nav(sel, count, code);
                        play_menu_sfx(music, Sfx::MenuMove);
                    }
                    KeyCode::Left | KeyCode::Right => match sel {
                        0 => {
                            if let Some(m) = music.as_mut() {
                                m.toggle_bgm();
                                m.play_sfx(Sfx::MenuMove);
                            }
                        }
                        1 => {
                            if let Some(m) = music.as_mut() {
                                m.toggle_sfx();
                                m.play_sfx(Sfx::MenuMove);
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Enter => match sel {
                        0 => {
                            if let Some(m) = music.as_mut() {
                                m.toggle_bgm();
                                m.play_sfx(Sfx::MenuMove);
                            }
                        }
                        1 => {
                            if let Some(m) = music.as_mut() {
                                m.toggle_sfx();
                                m.play_sfx(Sfx::MenuMove);
                            }
                        }
                        2 => {
                            play_menu_sfx(music, Sfx::MenuBack);
                            return Ok(());
                        }
                        _ => {}
                    },
                    KeyCode::Esc => {
                        play_menu_sfx(music, Sfx::MenuBack);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    let mc: usize = match mode {
        GameMode::Marathon => 3,
        GameMode::Endless => 2,
        GameMode::Sprint | GameMode::Ultra | GameMode::Versus => 1,
    };
    let count = mc + 11;
    let idx_bgm = mc + 8;
    let idx_sfx = mc + 9;
    let idx_back = mc + 10;

    let is_toggle = |s: usize| -> bool {
        s > mc + 2 && s <= mc + 7
    };

    loop {
        let (bgm_on, sfx_on) = match music.as_ref() {
            Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
            None => (false, false),
        };
        render::draw_settings(stdout, settings, mode, bgm_on, sfx_on, sel, false)?;
        if let Some(code) = read_key()? {
            match code {
                KeyCode::Up | KeyCode::Down => {
                    sel = menu_nav(sel, count, code);
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Left | KeyCode::Right => {
                    let dir = if code == KeyCode::Left { -1 } else { 1 };
                    if sel < idx_bgm {
                        adjust_setting(settings, sel, dir, mode);
                        play_menu_sfx(music, Sfx::MenuMove);
                    } else if sel == idx_bgm {
                        if let Some(m) = music.as_mut() {
                            m.toggle_bgm();
                            m.play_sfx(Sfx::MenuMove);
                        }
                    } else if sel == idx_sfx {
                        if let Some(m) = music.as_mut() {
                            m.toggle_sfx();
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Enter => {
                    if is_toggle(sel) {
                        adjust_setting(settings, sel, 0, mode);
                        play_menu_sfx(music, Sfx::MenuMove);
                    } else if sel == idx_bgm {
                        if let Some(m) = music.as_mut() {
                            m.toggle_bgm();
                            m.play_sfx(Sfx::MenuMove);
                        }
                    } else if sel == idx_sfx {
                        if let Some(m) = music.as_mut() {
                            m.toggle_sfx();
                            m.play_sfx(Sfx::MenuMove);
                        }
                    } else if sel == idx_back {
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
