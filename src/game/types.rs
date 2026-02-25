use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    Marathon,
    Sprint,
    Ultra,
    Endless,
}

pub const LINE_CLEAR_ANIM_DURATION: Duration = Duration::from_millis(300);
pub const ARE_DELAY: Duration = Duration::from_millis(100);

pub struct LineClearAnimation {
    pub rows: Vec<usize>,
    pub started_at: Instant,
    pub phase: u8,
}

impl LineClearAnimation {
    pub fn new(rows: Vec<usize>) -> Self {
        Self {
            rows,
            started_at: Instant::now(),
            phase: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LastMove {
    None,
    Rotate,
    Move,
}

#[derive(Clone)]
pub struct ClearAction {
    pub label: String,
    pub points: u32,
}
