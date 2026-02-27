use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use std::time::{Duration, Instant};

use crate::audio::{self, Sfx};
use crate::game::{Game, LastMove, ARE_DELAY};

fn force_quit() -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, "force quit")
}

pub(crate) fn read_key() -> io::Result<Option<KeyCode>> {
    if let Event::Key(KeyEvent { code, kind, modifiers, .. }) = event::read()? {
        if kind != KeyEventKind::Press {
            return Ok(None);
        }
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            return Err(force_quit());
        }
        Ok(Some(code))
    } else {
        Ok(None)
    }
}

pub(crate) const DAS_DELAY: Duration = Duration::from_millis(167);
pub(crate) const ARR_INTERVAL: Duration = Duration::from_millis(33);
pub(crate) const DAS_RELEASE: Duration = Duration::from_millis(100);

pub(crate) struct DasState {
    pub direction: i32,
    pub start: Instant,
    pub charged: bool,
    pub last_arr_move: Instant,
    pub last_event: Instant,
}

impl DasState {
    pub fn new(direction: i32) -> Self {
        let now = Instant::now();
        Self {
            direction,
            start: now,
            charged: false,
            last_arr_move: now,
            last_event: now,
        }
    }

    pub fn reset_timers(&mut self) {
        let now = Instant::now();
        self.last_event = now;
        self.start = now;
        self.last_arr_move = now;
    }
}

pub(crate) fn play_clear_sfx(music: &audio::MusicPlayer, game: &Game, prev_level: u32) {
    music.play_sfx(Sfx::Lock);

    if let Some(anim) = &game.line_clear_anim {
        let lines = anim.rows.len() as u32;
        if let Some(action) = &game.last_action {
            let label = &action.label;
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

pub(crate) struct InputState {
    pub das: Option<DasState>,
    pub irs: Option<i32>,
    pub ihs: bool,
    pub last_tick: Instant,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            das: None,
            irs: None,
            ihs: false,
            last_tick: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.das = None;
        self.irs = None;
        self.ihs = false;
        self.last_tick = Instant::now();
    }
}

fn play_move_sfx(music: &Option<audio::MusicPlayer>) {
    if let Some(m) = music.as_ref() {
        m.play_sfx(Sfx::Move);
    }
}

/// Handle a game key press. Returns true if a hard-drop occurred (caller may need post-lock logic).
pub(crate) fn handle_game_key(
    code: KeyCode,
    game: &mut Game,
    input: &mut InputState,
    music: &Option<audio::MusicPlayer>,
) -> bool {
    match code {
        KeyCode::Left | KeyCode::Right => {
            let dir = if code == KeyCode::Left { -1 } else { 1 };
            if let Some(d) = input.das.as_mut().filter(|d| d.direction == dir) {
                d.last_event = Instant::now();
            } else {
                input.das = Some(DasState::new(dir));
                if !game.in_are() && game.move_piece(0, dir) {
                    play_move_sfx(music);
                }
            }
            false
        }
        KeyCode::Down => {
            if !game.in_are() {
                game.soft_drop();
            }
            false
        }
        KeyCode::Up | KeyCode::Char('x') | KeyCode::Char('X') => {
            if game.in_are() {
                input.irs = Some(1);
            } else {
                game.rotate_cw();
                if game.last_move == LastMove::Rotate {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::Rotate);
                    }
                }
            }
            false
        }
        KeyCode::Char('z') | KeyCode::Char('Z') => {
            if game.in_are() {
                input.irs = Some(-1);
            } else {
                game.rotate_ccw();
                if game.last_move == LastMove::Rotate {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::Rotate);
                    }
                }
            }
            false
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if game.in_are() {
                input.ihs = true;
            } else {
                let was_used = game.hold_used;
                game.hold_piece();
                if !was_used && game.hold_used {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::Hold);
                    }
                }
            }
            false
        }
        KeyCode::Char(' ') => {
            if !game.in_are() {
                let prev_level = game.level;
                if let Some(m) = music.as_ref() {
                    m.play_sfx(Sfx::HardDrop);
                }
                game.hard_drop();
                if let Some(m) = music.as_ref() {
                    play_clear_sfx(m, game, prev_level);
                }
                input.last_tick = Instant::now();
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Run DAS auto-repeat, ARE check (IRS/IHS), lock delay, gravity tick.
/// Returns true if a lock occurred (caller may need post-lock logic).
pub(crate) fn update_game_timers(
    game: &mut Game,
    input: &mut InputState,
    music: &Option<audio::MusicPlayer>,
) -> bool {
    // DAS release
    if let Some(d) = &input.das {
        if d.last_event.elapsed() >= DAS_RELEASE {
            input.das = None;
        }
    }

    // DAS auto-repeat
    if let Some(d) = &mut input.das {
        if !game.in_are() {
            if !d.charged && d.start.elapsed() >= DAS_DELAY {
                d.charged = true;
                d.last_arr_move = Instant::now();
                if game.move_piece(0, d.direction) {
                    play_move_sfx(music);
                }
            } else if d.charged && d.last_arr_move.elapsed() >= ARR_INTERVAL {
                d.last_arr_move = Instant::now();
                if game.move_piece(0, d.direction) {
                    play_move_sfx(music);
                }
            }
        }
    }

    // ARE check with IRS/IHS
    if game.in_are() {
        if game.check_are() {
            if input.ihs {
                input.ihs = false;
                let was_used = game.hold_used;
                game.hold_piece();
                if !was_used && game.hold_used {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::Hold);
                    }
                }
            }
            if let Some(dir) = input.irs.take() {
                if dir > 0 {
                    game.rotate_cw();
                } else {
                    game.rotate_ccw();
                }
                if game.last_move == LastMove::Rotate {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::Rotate);
                    }
                }
            }
            input.last_tick = Instant::now();
            if let Some(d) = &mut input.das {
                if d.charged {
                    while game.move_piece(0, d.direction) {}
                    d.last_arr_move = Instant::now();
                }
            }
        }
        return false;
    }

    // Lock delay
    if let Some(lock_start) = game.lock_delay {
        if lock_start.elapsed() >= game.lock_delay_duration() {
            game.lock_delay = None;
            let prev_level = game.level;
            game.lock_and_begin_clear();
            if let Some(m) = music.as_ref() {
                play_clear_sfx(m, game, prev_level);
            }
            input.last_tick = Instant::now();
            return true;
        }
    }

    // Gravity tick
    if input.last_tick.elapsed() >= game.drop_interval() {
        game.tick();
        input.last_tick = Instant::now();
    }

    false
}

/// Compute the poll timeout based on game state + DAS state.
pub(crate) fn compute_timeout(game: &Game, input: &InputState) -> Duration {
    let mut timeout = if game.in_are() {
        Duration::from_secs(1)
    } else {
        let gravity_remaining = game.drop_interval().saturating_sub(input.last_tick.elapsed());
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

    if let Some(d) = &input.das {
        timeout = timeout.min(DAS_RELEASE.saturating_sub(d.last_event.elapsed()));
        if !d.charged {
            timeout = timeout.min(DAS_DELAY.saturating_sub(d.start.elapsed()));
        } else {
            timeout = timeout.min(ARR_INTERVAL.saturating_sub(d.last_arr_move.elapsed()));
        }
    }

    timeout
}

pub(crate) fn play_menu_sfx(music: &Option<audio::MusicPlayer>, sfx: Sfx) {
    if let Some(m) = music.as_ref() {
        m.play_sfx(sfx);
    }
}

pub(crate) fn toggle_bgm(music: &mut Option<audio::MusicPlayer>) {
    if let Some(m) = music.as_mut() {
        m.toggle_bgm();
        m.play_sfx(Sfx::MenuMove);
    }
}

pub(crate) fn toggle_sfx(music: &mut Option<audio::MusicPlayer>) {
    if let Some(m) = music.as_mut() {
        m.toggle_sfx();
        m.play_sfx(Sfx::MenuMove);
    }
}

pub(crate) fn menu_nav(sel: usize, count: usize, code: KeyCode) -> usize {
    match code {
        KeyCode::Up => sel.checked_sub(1).unwrap_or(count - 1),
        KeyCode::Down => (sel + 1) % count,
        _ => sel,
    }
}
