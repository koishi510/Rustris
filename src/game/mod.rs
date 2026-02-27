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
    pub srs_enabled: bool,
    pub hold_enabled: bool,
    pub lock_delay_ms: u32,
    pub move_reset: Option<u32>,
    pub move_reset_count: u32,
    pub garbage_rise_anim: Option<GarbageRiseAnimation>,
}
