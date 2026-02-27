mod animation;
mod board;
mod movement;
mod scoring;

pub mod garbage;
pub mod piece;
pub mod records;
pub mod settings;
pub mod types;
pub use types::*;

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use crate::game::piece::*;
use crate::game::settings::Settings;

pub struct Game {
    pub board: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current: Piece,
    pub next_queue: Vec<usize>,
    pub hold: Option<usize>,
    pub hold_used: bool,
    bag: Bag,
    pub score: u32,
    pub lines: u32,
    pub level: u32,
    pub start_level: u32,
    pub game_over: bool,
    pub last_move: LastMove,
    pub combo: i32,
    pub back_to_back: bool,
    pub last_action: Option<ClearAction>,
    pub last_action_time: Instant,
    pub lock_delay: Option<Instant>,
    pub line_clear_anim: Option<LineClearAnimation>,
    pub are_timer: Option<Instant>,
    pub mode: GameMode,
    pub game_start: Instant,
    pub elapsed: Duration,
    pub cleared: bool,
    pub marathon_goal: u32,
    pub sprint_goal: u32,
    pub ultra_time: u32,
    pub level_cap: Option<u32>,
    pub ghost_enabled: bool,
    pub line_clear_anim_enabled: bool,
    pub next_count: usize,
    pub srs: bool,
    pub hold_enabled: bool,
    pub lock_delay_ms: u32,
    pub move_reset: Option<u32>,
    pub move_reset_count: u32,
    pub garbage_rise_anim: Option<GarbageRiseAnimation>,
}

impl Game {
    pub fn new(mode: GameMode, settings: &Settings) -> Self {
        let start_level = match mode {
            GameMode::Marathon | GameMode::Endless => settings.level,
            GameMode::Sprint | GameMode::Ultra => 1,
            GameMode::Versus => settings.level,
        };
        let mut bag = Bag::new(settings.bag_randomizer);
        let current_kind = bag.next();
        let mut next_queue = Vec::with_capacity(settings.next_count);
        for _ in 0..settings.next_count {
            next_queue.push(bag.next());
        }
        Self {
            board: [[EMPTY; BOARD_WIDTH]; BOARD_HEIGHT],
            current: Piece::new(current_kind),
            next_queue,
            hold: None,
            hold_used: false,
            bag,
            score: 0,
            lines: 0,
            level: start_level,
            start_level,
            game_over: false,
            last_move: LastMove::None,
            combo: -1,
            back_to_back: false,
            last_action: None,
            last_action_time: Instant::now(),
            lock_delay: None,
            line_clear_anim: None,
            are_timer: None,
            mode,
            game_start: Instant::now(),
            elapsed: Duration::ZERO,
            cleared: false,
            marathon_goal: settings.marathon_goal,
            sprint_goal: settings.sprint_goal,
            ultra_time: settings.ultra_time,
            level_cap: settings.level_cap,
            ghost_enabled: settings.ghost,
            line_clear_anim_enabled: settings.line_clear_anim,
            next_count: settings.next_count,
            srs: settings.srs,
            hold_enabled: settings.hold_enabled,
            lock_delay_ms: settings.lock_delay_ms,
            move_reset: settings.move_reset,
            move_reset_count: 0,
            garbage_rise_anim: None,
        }
    }
}
