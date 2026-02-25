use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::io;

use crate::audio::{self, Sfx};
use crate::game::GameMode;
use crate::piece::MAX_NEXT_COUNT;
use crate::records::Records;
use crate::render;
use crate::settings::Settings;

pub(crate) fn adjust_setting(settings: &mut Settings, sel: usize, direction: i32, mode: GameMode) {
    let mc: usize = match mode {
        GameMode::Marathon => 3,
        GameMode::Endless => 2,
        GameMode::Sprint | GameMode::Ultra => 1,
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
                    match (settings.level_cap, direction) {
                        (Some(c), 1) if c >= 20 => settings.level_cap = None,
                        (Some(c), 1) => settings.level_cap = Some((c + 1).min(20)),
                        (Some(c), -1) => settings.level_cap = Some(if c <= 1 { 1 } else { c - 1 }),
                        (None, -1) => settings.level_cap = Some(20),
                        _ => {}
                    }
                }
                _ => {}
            },
            GameMode::Endless => match sel {
                0 => {
                    let v = settings.level as i32 + direction;
                    settings.level = v.clamp(1, 20) as u32;
                }
                1 => {
                    match (settings.level_cap, direction) {
                        (Some(c), 1) if c >= 20 => settings.level_cap = None,
                        (Some(c), 1) => settings.level_cap = Some((c + 1).min(20)),
                        (Some(c), -1) => settings.level_cap = Some(if c <= 1 { 1 } else { c - 1 }),
                        (None, -1) => settings.level_cap = Some(20),
                        _ => {}
                    }
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
            (Some(n), -1) if n == 0 => {}
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
        settings.srs = !settings.srs;
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
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        sel = sel.checked_sub(1).unwrap_or(count - 1);
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                    KeyCode::Down => {
                        sel = (sel + 1) % count;
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
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
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuBack);
                            }
                            return Ok(());
                        }
                        _ => {}
                    },
                    KeyCode::Esc => {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuBack);
                        }
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
        GameMode::Sprint | GameMode::Ultra => 1,
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
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    sel = sel.checked_sub(1).unwrap_or(count - 1);
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Down => {
                    sel = (sel + 1) % count;
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Left | KeyCode::Right => {
                    let dir = if code == KeyCode::Left { -1 } else { 1 };
                    if sel < idx_bgm {
                        adjust_setting(settings, sel, dir, mode);
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
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
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
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
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuBack);
                        }
                        return Ok(());
                    }
                }
                KeyCode::Esc => {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuBack);
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

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

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    sel = sel.checked_sub(1).unwrap_or(count - 1);
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Down => {
                    sel = (sel + 1) % count;
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Left => {
                    if sel == 0 {
                        mode = match mode {
                            GameMode::Marathon => GameMode::Endless,
                            GameMode::Sprint => GameMode::Marathon,
                            GameMode::Ultra => GameMode::Sprint,
                            GameMode::Endless => GameMode::Ultra,
                        };
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Right => {
                    if sel == 0 {
                        mode = match mode {
                            GameMode::Marathon => GameMode::Sprint,
                            GameMode::Sprint => GameMode::Ultra,
                            GameMode::Ultra => GameMode::Endless,
                            GameMode::Endless => GameMode::Marathon,
                        };
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Enter => {
                    if sel == 1 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        return Ok(Some(mode));
                    } else if sel == 2 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        run_settings(stdout, music, settings, mode, false)?;
                    } else if sel == 3 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        run_records(stdout, music, records, mode)?;
                    } else if sel == 4 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        render::draw_help(stdout, 0)?;
                        loop {
                            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                if code == KeyCode::Enter || code == KeyCode::Esc {
                                    if let Some(m) = music.as_ref() {
                                        m.play_sfx(Sfx::MenuBack);
                                    }
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
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    sel = sel.checked_sub(1).unwrap_or(1);
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Down => {
                    sel = 1;
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Left => {
                    if sel == 0 {
                        mode = match mode {
                            GameMode::Marathon => GameMode::Endless,
                            GameMode::Sprint => GameMode::Marathon,
                            GameMode::Ultra => GameMode::Sprint,
                            GameMode::Endless => GameMode::Ultra,
                        };
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Right => {
                    if sel == 0 {
                        mode = match mode {
                            GameMode::Marathon => GameMode::Sprint,
                            GameMode::Sprint => GameMode::Ultra,
                            GameMode::Ultra => GameMode::Endless,
                            GameMode::Endless => GameMode::Marathon,
                        };
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Enter => {
                    if sel == 1 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuBack);
                        }
                        return Ok(());
                    }
                }
                KeyCode::Esc => {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuBack);
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
