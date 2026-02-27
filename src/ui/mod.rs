mod game_loop;
pub mod input;
mod menus;
pub mod versus;

pub use game_loop::run_game;
pub use menus::{select_mode, run_versus_menu, VersusAction};

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::audio::{self, Sfx};
use crate::game::Game;

pub(crate) fn force_quit() -> io::Error {
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

pub(crate) fn play_menu_sfx(music: &Option<audio::MusicPlayer>, sfx: Sfx) {
    if let Some(m) = music.as_ref() {
        m.play_sfx(sfx);
    }
}

pub(crate) fn menu_nav(sel: usize, count: usize, code: KeyCode) -> usize {
    match code {
        KeyCode::Up => sel.checked_sub(1).unwrap_or(count - 1),
        KeyCode::Down => (sel + 1) % count,
        _ => sel,
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
