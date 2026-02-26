use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal,
};
use std::io;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::audio::{self, Sfx};
use crate::game::{Game, GameMode};
use crate::game::records;
use crate::render;
use crate::game::settings::Settings;

use super::input::{self, InputState};
use super::menus::run_settings;
use super::{menu_nav, play_menu_sfx};

fn iso8601_now() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days: [i64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut m = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md {
            m = i;
            break;
        }
        remaining -= md;
    }
    let d = remaining + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        d,
        hours,
        minutes,
        seconds
    )
}

pub fn run_game(
    stdout: &mut io::Stdout,
    mode: GameMode,
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
    records: &mut records::Records,
) -> io::Result<bool> {
    let mut game = Game::new(mode, settings);
    let mut inp = InputState::new();
    if let Some(m) = music.as_mut() {
        m.play();
    }
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    loop {
        if game.game_over {
            if let Some(m) = music.as_mut() {
                m.stop();
                if game.cleared {
                    m.play_sfx(Sfx::Clear);
                } else {
                    m.play_sfx(Sfx::GameOver);
                }
            }

            let time_ms = Some(game.elapsed.as_millis() as u64);
            let now = iso8601_now();
            let record = records::ScoreRecord {
                score: game.score,
                lines: game.lines,
                level: game.level,
                time: time_ms,
                date: now,
            };
            let valid_for_record = match mode {
                GameMode::Marathon => settings.marathon_goal == 150,
                GameMode::Sprint => game.cleared && settings.sprint_goal == 40,
                GameMode::Ultra => settings.ultra_time == 120,
                GameMode::Endless => true,
                GameMode::Versus => false,
            };
            let rank = if valid_for_record {
                let r = records.add(mode, record);
                records.save();
                r
            } else {
                None
            };

            let mut sel: usize = 0;
            let count: usize = 3;
            loop {
                render::draw_game_over(stdout, &game, sel, rank)?;
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Up | KeyCode::Down => {
                            sel = menu_nav(sel, count, code);
                            play_menu_sfx(music, Sfx::MenuMove);
                        }
                        KeyCode::Enter => match sel {
                            0 => {
                                play_menu_sfx(music, Sfx::MenuSelect);
                                break;
                            }
                            1 => {
                                play_menu_sfx(music, Sfx::MenuSelect);
                                return Ok(false);
                            }
                            2 => {
                                return Ok(true);
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            game = Game::new(mode, settings);
            inp.reset();
            if let Some(m) = music.as_mut() {
                m.play();
            }
            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
            continue;
        }

        game.update_elapsed();
        if game.mode == GameMode::Ultra && game.elapsed >= Duration::from_secs(game.ultra_time as u64) {
            game.game_over = true;
        }

        render::draw(stdout, &game)?;

        if game.is_animating() {
            if game.update_animation() {
                if event::poll(Duration::from_millis(16))? {
                    if let Event::Key(_) = event::read()? {}
                }
                continue;
            } else {
                game.finish_clear();
                inp.last_tick = Instant::now();
                continue;
            }
        }

        let mut timeout = input::compute_timeout(&game, &inp);

        if let Some(remaining) = game.time_remaining() {
            timeout = timeout.min(remaining);
        }

        if game.mode == GameMode::Sprint || game.mode == GameMode::Ultra {
            timeout = timeout.min(Duration::from_millis(32));
        }

        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('P') => {
                        if let Some(m) = music.as_mut() {
                            m.play_sfx(Sfx::Pause);
                            m.pause();
                        }
                        let mut sel: usize = 0;
                        let count: usize = 6;
                        let mut retry = false;
                        loop {
                            render::draw_pause(stdout, sel)?;
                            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                match code {
                                    KeyCode::Up | KeyCode::Down => {
                                        sel = menu_nav(sel, count, code);
                                        play_menu_sfx(music, Sfx::MenuMove);
                                    }
                                    KeyCode::Enter => match sel {
                                        0 => {
                                            if let Some(m) = music.as_ref() {
                                                m.play_sfx(Sfx::Resume);
                                            }
                                            break;
                                        }
                                        1 => {
                                            play_menu_sfx(music, Sfx::MenuSelect);
                                            run_settings(stdout, music, settings, mode, true)?;
                                        }
                                        2 => {
                                            play_menu_sfx(music, Sfx::MenuSelect);
                                            render::draw_help(stdout, 0)?;
                                            loop {
                                                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                                    if code == KeyCode::Enter || code == KeyCode::Esc {
                                                        play_menu_sfx(music, Sfx::MenuBack);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        3 => {
                                            retry = true;
                                            break;
                                        }
                                        4 => {
                                            if let Some(m) = music.as_mut() {
                                                m.play_sfx(Sfx::MenuBack);
                                                m.stop();
                                            }
                                            return Ok(false);
                                        }
                                        5 => {
                                            if let Some(m) = music.as_mut() {
                                                m.stop();
                                            }
                                            return Ok(true);
                                        }
                                        _ => {}
                                    },
                                    KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('P') => {
                                        if let Some(m) = music.as_ref() {
                                            m.play_sfx(Sfx::Resume);
                                        }
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if retry {
                            play_menu_sfx(music, Sfx::MenuSelect);
                            game = Game::new(mode, settings);
                            inp.reset();
                            if let Some(m) = music.as_mut() {
                                m.play();
                            }
                            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                            continue;
                        }
                        if let Some(m) = music.as_mut() {
                            m.resume();
                        }
                        game.reset_game_start();
                        inp.last_tick = Instant::now();
                        if game.lock_delay.is_some() {
                            game.lock_delay = Some(Instant::now());
                        }
                        if let Some(d) = &mut inp.das {
                            let now = Instant::now();
                            d.last_event = now;
                            d.start = now;
                            d.last_arr_move = now;
                        }
                        continue;
                    }
                    other => {
                        input::handle_game_key(other, &mut game, &mut inp, music);
                    }
                }
            }
        }

        if game.in_are() {
            input::update_game_timers(&mut game, &mut inp, music);
            continue;
        }

        input::update_game_timers(&mut game, &mut inp, music);
    }
}
