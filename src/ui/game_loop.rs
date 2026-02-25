use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal,
};
use std::io;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::audio::{self, Sfx};
use crate::game::{Game, GameMode, LastMove, ARE_DELAY};
use crate::records;
use crate::render;
use crate::settings::Settings;

use super::menus::run_settings;

const DAS_DELAY: Duration = Duration::from_millis(167);
const ARR_INTERVAL: Duration = Duration::from_millis(33);
const DAS_RELEASE: Duration = Duration::from_millis(100);

struct DasState {
    direction: i32,
    start: Instant,
    charged: bool,
    last_arr_move: Instant,
    last_event: Instant,
}

impl DasState {
    fn new(direction: i32) -> Self {
        let now = Instant::now();
        Self {
            direction,
            start: now,
            charged: false,
            last_arr_move: now,
            last_event: now,
        }
    }
}

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

    // Days since epoch to Y-M-D (simplified leap year calculation)
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

fn play_clear_sfx(music: &audio::MusicPlayer, game: &Game, prev_level: u32) {
    music.play_sfx(Sfx::Lock);

    if let Some(anim) = &game.line_clear_anim {
        let lines = anim.rows.len() as u32;
        if let Some(action) = &game.last_action {
            let label = &action.label;
            // Priority from low to high (last call wins)
            if label.contains("T-Spin") || label.contains("Mini T-Spin") {
                music.play_sfx(Sfx::TSpinClear(lines));
            } else {
                music.play_sfx(Sfx::LineClear(lines));
            }
            if label.contains("Combo") {
                music.play_sfx(Sfx::Combo(game.combo as u32));
            }
            if label.contains("B2B") {
                music.play_sfx(Sfx::BackToBack);
            }
            if label.contains("ALL CLEAR") {
                music.play_sfx(Sfx::AllClear);
            }
        }
    } else if !game.game_over {
        if let Some(action) = &game.last_action {
            if game.last_action_time.elapsed().as_millis() < 100 {
                if action.label.contains("Mini T-Spin") {
                    music.play_sfx(Sfx::TSpinMini);
                } else if action.label.contains("T-Spin") {
                    music.play_sfx(Sfx::TSpin);
                }
            }
        }
    }

    if game.level > prev_level {
        music.play_sfx(Sfx::LevelUp);
    }
}

pub fn run_game(
    stdout: &mut io::Stdout,
    mode: GameMode,
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
    records: &mut records::Records,
) -> io::Result<bool> {
    let mut game = Game::new(mode, settings);
    let mut last_tick = Instant::now();
    let mut das: Option<DasState> = None;
    if let Some(m) = music.as_mut() {
        m.play();
    }
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    let play_move_sfx = |music: &Option<audio::MusicPlayer>| {
        if let Some(m) = music.as_ref() {
            m.play_sfx(Sfx::Move);
        }
    };

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
                        KeyCode::Enter => match sel {
                            0 => {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::MenuSelect);
                                }
                                break;
                            }
                            1 => {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::MenuSelect);
                                }
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
            last_tick = Instant::now();
            das = None;
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
                last_tick = Instant::now();
                continue;
            }
        }

        let mut timeout = if game.in_are() {
            Duration::from_secs(1)
        } else {
            let gravity_remaining = game.drop_interval().saturating_sub(last_tick.elapsed());
            if let Some(lock_start) = game.lock_delay {
                let lock_remaining = game.lock_delay_duration().saturating_sub(lock_start.elapsed());
                gravity_remaining.min(lock_remaining)
            } else {
                gravity_remaining
            }
        };

        if let Some(are_start) = game.are_timer {
            timeout = timeout.min(ARE_DELAY.saturating_sub(are_start.elapsed()));
        }

        if let Some(d) = &das {
            timeout = timeout.min(DAS_RELEASE.saturating_sub(d.last_event.elapsed()));
            if !d.charged {
                timeout = timeout.min(DAS_DELAY.saturating_sub(d.start.elapsed()));
            } else {
                timeout = timeout.min(ARR_INTERVAL.saturating_sub(d.last_arr_move.elapsed()));
            }
        }

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
                                    KeyCode::Enter => match sel {
                                        0 => {
                                            if let Some(m) = music.as_ref() {
                                                m.play_sfx(Sfx::Resume);
                                            }
                                            break;
                                        }
                                        1 => {
                                            if let Some(m) = music.as_ref() {
                                                m.play_sfx(Sfx::MenuSelect);
                                            }
                                            run_settings(stdout, music, settings, mode, true)?;
                                        }
                                        2 => {
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
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
                            game = Game::new(mode, settings);
                            last_tick = Instant::now();
                            das = None;
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
                        last_tick = Instant::now();
                        if game.lock_delay.is_some() {
                            game.lock_delay = Some(Instant::now());
                        }
                        if let Some(d) = &mut das {
                            let now = Instant::now();
                            d.last_event = now;
                            d.start = now;
                            d.last_arr_move = now;
                        }
                        continue;
                    }
                    KeyCode::Left | KeyCode::Right => {
                        let dir = if code == KeyCode::Left { -1 } else { 1 };
                        if das.as_ref().map_or(false, |d| d.direction == dir) {
                            das.as_mut().unwrap().last_event = Instant::now();
                        } else {
                            das = Some(DasState::new(dir));
                            if !game.in_are() {
                                if game.move_piece(0, dir) {
                                    play_move_sfx(&music);
                                }
                            }
                        }
                    }
                    KeyCode::Down => {
                        if !game.in_are() {
                            game.soft_drop();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('x') | KeyCode::Char('X') => {
                        if !game.in_are() {
                            game.rotate_cw();
                            if game.last_move == LastMove::Rotate {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::Rotate);
                                }
                            }
                        }
                    }
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        if !game.in_are() {
                            game.rotate_ccw();
                            if game.last_move == LastMove::Rotate {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::Rotate);
                                }
                            }
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if !game.in_are() {
                            let was_used = game.hold_used;
                            game.hold_piece();
                            if !was_used && game.hold_used {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::Hold);
                                }
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        if !game.in_are() {
                            let prev_level = game.level;
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::HardDrop);
                            }
                            game.hard_drop();
                            if let Some(m) = music.as_ref() {
                                play_clear_sfx(m, &game, prev_level);
                            }
                            last_tick = Instant::now();
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(d) = &das {
            if d.last_event.elapsed() >= DAS_RELEASE {
                das = None;
            }
        }

        if let Some(d) = &mut das {
            if !game.in_are() {
                if !d.charged && d.start.elapsed() >= DAS_DELAY {
                    d.charged = true;
                    d.last_arr_move = Instant::now();
                    if game.move_piece(0, d.direction) {
                        play_move_sfx(&music);
                    }
                } else if d.charged && d.last_arr_move.elapsed() >= ARR_INTERVAL {
                    d.last_arr_move = Instant::now();
                    if game.move_piece(0, d.direction) {
                        play_move_sfx(&music);
                    }
                }
            }
        }

        if game.in_are() {
            if game.check_are() {
                last_tick = Instant::now();
                if let Some(d) = &mut das {
                    if d.charged {
                        while game.move_piece(0, d.direction) {}
                        d.last_arr_move = Instant::now();
                    }
                }
            }
            continue;
        }

        if let Some(lock_start) = game.lock_delay {
            if lock_start.elapsed() >= game.lock_delay_duration() {
                game.lock_delay = None;
                let prev_level = game.level;
                game.lock_and_begin_clear();
                if let Some(m) = music.as_ref() {
                    play_clear_sfx(m, &game, prev_level);
                }
                last_tick = Instant::now();
                continue;
            }
        }

        if last_tick.elapsed() >= game.drop_interval() {
            game.tick();
            last_tick = Instant::now();
        }
    }
}
