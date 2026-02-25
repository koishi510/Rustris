use std::time::{Duration, Instant};

use crate::piece::*;
use crate::settings::Settings;

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    Marathon,
    Sprint,
    Ultra,
    Endless,
}

pub const LOCK_DELAY: Duration = Duration::from_millis(500);
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
}

impl Game {
    pub fn new(mode: GameMode, settings: &Settings) -> Self {
        let start_level = match mode {
            GameMode::Marathon | GameMode::Endless => settings.level,
            GameMode::Sprint | GameMode::Ultra => 1,
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
        }
    }

    fn pop_next(&mut self) -> Piece {
        let kind = self.next_queue.remove(0);
        self.next_queue.push(self.bag.next());
        Piece::new(kind)
    }

    fn is_occupied(&self, r: i32, c: i32) -> bool {
        if r < 0 || r >= BOARD_HEIGHT as i32 || c < 0 || c >= BOARD_WIDTH as i32 {
            return true;
        }
        self.board[r as usize][c as usize] != EMPTY
    }

    pub fn fits(&self, piece: &Piece) -> bool {
        for (r, c) in piece.cells() {
            if c < 0 || c >= BOARD_WIDTH as i32 || r >= BOARD_HEIGHT as i32 {
                return false;
            }
            if r >= 0 && self.board[r as usize][c as usize] != EMPTY {
                return false;
            }
        }
        true
    }

    fn detect_tspin(&self) -> (bool, bool) {
        if self.current.kind != KIND_T || self.last_move != LastMove::Rotate {
            return (false, false);
        }

        let rot = self.current.rotation as usize;
        let cr = self.current.row;
        let cc = self.current.col;

        let front = &T_FRONT_CORNERS[rot];
        let back = &T_BACK_CORNERS[rot];

        let front_occupied = front
            .iter()
            .filter(|&&[dr, dc]| self.is_occupied(cr + dr, cc + dc))
            .count();
        let back_occupied = back
            .iter()
            .filter(|&&[dr, dc]| self.is_occupied(cr + dr, cc + dc))
            .count();

        let total = front_occupied + back_occupied;
        if total < 3 {
            return (false, false);
        }

        // Full T-Spin if both front corners occupied, otherwise Mini
        if front_occupied == 2 {
            (true, false)
        } else {
            (true, true)
        }
    }

    fn lock_current(&mut self) {
        let color_id = (self.current.kind + 1) as u8;
        for (r, c) in self.current.cells() {
            if r >= 0 && r < BOARD_HEIGHT as i32 && c >= 0 && c < BOARD_WIDTH as i32 {
                self.board[r as usize][c as usize] = color_id;
            }
        }
    }

    fn find_full_rows(&self) -> Vec<usize> {
        let mut rows = Vec::new();
        for r in 0..BOARD_HEIGHT {
            if self.board[r].iter().all(|&c| c != EMPTY) {
                rows.push(r);
            }
        }
        rows
    }

    fn remove_rows(&mut self, rows: &[usize]) {
        let mut new_board = [[EMPTY; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut dest = BOARD_HEIGHT - 1;
        for src in (0..BOARD_HEIGHT).rev() {
            if rows.contains(&src) {
                continue;
            }
            new_board[dest] = self.board[src];
            dest = dest.wrapping_sub(1);
        }
        self.board = new_board;
    }

    fn spawn_next(&mut self) {
        self.current = self.pop_next();
        self.hold_used = false;
        self.last_move = LastMove::None;
        self.lock_delay = None;
        if !self.fits(&self.current) {
            self.game_over = true;
        }
    }

    pub fn hold_piece(&mut self) {
        if self.hold_used {
            return;
        }
        self.hold_used = true;
        self.lock_delay = None;
        let cur_kind = self.current.kind;
        match self.hold {
            Some(held_kind) => {
                self.hold = Some(cur_kind);
                self.current = Piece::new(held_kind);
                if !self.fits(&self.current) {
                    self.game_over = true;
                }
            }
            None => {
                self.hold = Some(cur_kind);
                self.current = self.pop_next();
                if !self.fits(&self.current) {
                    self.game_over = true;
                }
            }
        }
        self.last_move = LastMove::None;
    }

    fn is_on_ground(&self) -> bool {
        let mut test = self.current.clone();
        test.row += 1;
        !self.fits(&test)
    }

    pub fn move_piece(&mut self, dr: i32, dc: i32) -> bool {
        let mut moved = self.current.clone();
        moved.row += dr;
        moved.col += dc;
        if self.fits(&moved) {
            self.current = moved;
            self.last_move = LastMove::Move;
            if self.lock_delay.is_some() {
                if self.is_on_ground() {
                    self.lock_delay = Some(Instant::now());
                } else {
                    self.lock_delay = None;
                }
            }
            true
        } else {
            false
        }
    }

    fn try_rotate(&mut self, new_rotation: u8) {
        if self.current.kind == 1 {
            return;
        }

        let from = self.current.rotation;
        let to = new_rotation;
        let kick_table = if self.current.kind == 0 {
            &KICK_I
        } else {
            &KICK_JLTSZ
        };
        let idx = kick_index(from, to);
        let kicks = &kick_table[idx];

        for &[dc, dr] in kicks {
            let mut test = self.current.clone();
            test.rotation = to;
            test.col += dc;
            test.row += dr;
            if self.fits(&test) {
                self.current = test;
                self.last_move = LastMove::Rotate;
                if self.lock_delay.is_some() {
                    if self.is_on_ground() {
                        self.lock_delay = Some(Instant::now());
                    } else {
                        self.lock_delay = None;
                    }
                }
                return;
            }
        }
    }

    pub fn rotate_cw(&mut self) {
        let new_rot = (self.current.rotation + 1) % 4;
        self.try_rotate(new_rot);
    }

    pub fn rotate_ccw(&mut self) {
        let new_rot = (self.current.rotation + 3) % 4;
        self.try_rotate(new_rot);
    }

    pub fn hard_drop(&mut self) -> u32 {
        let mut cells_dropped = 0u32;
        while self.move_piece(1, 0) {
            cells_dropped += 1;
        }
        let points = cells_dropped * 2;
        self.score += points;
        self.lock_delay = None;
        self.lock_and_begin_clear();
        points
    }

    pub fn soft_drop(&mut self) -> bool {
        if self.move_piece(1, 0) {
            self.score += 1;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self) {
        let g = self.gravity();
        if g >= 20.0 {
            while self.move_piece(1, 0) {}
            if self.lock_delay.is_none() {
                self.lock_delay = Some(Instant::now());
            }
        } else if g >= 1.0 {
            let rows = g.floor() as i32;
            for _ in 0..rows {
                if !self.move_piece(1, 0) {
                    if self.lock_delay.is_none() {
                        self.lock_delay = Some(Instant::now());
                    }
                    return;
                }
            }
        } else {
            if !self.move_piece(1, 0) {
                if self.lock_delay.is_none() {
                    self.lock_delay = Some(Instant::now());
                }
            }
        }
    }

    pub fn lock_and_begin_clear(&mut self) -> bool {
        let (is_tspin, is_mini) = self.detect_tspin();

        self.lock_current();
        let full_rows = self.find_full_rows();
        let cleared = full_rows.len() as u32;

        if cleared > 0 {
            self.lines += cleared;
            self.combo += 1;

            let is_difficult = cleared == 4 || is_tspin;

            let base = if is_tspin {
                if is_mini {
                    match cleared {
                        1 => 200,
                        2 => 400,
                        _ => 100,
                    }
                } else {
                    match cleared {
                        1 => 800,
                        2 => 1200,
                        3 => 1600,
                        _ => 400,
                    }
                }
            } else {
                match cleared {
                    1 => 100,
                    2 => 300,
                    3 => 500,
                    4 => 800,
                    _ => 0,
                }
            };

            let b2b_bonus = if is_difficult && self.back_to_back {
                base / 2
            } else {
                0
            };

            let line_points = (base + b2b_bonus) * self.level;

            let combo_points = if self.combo > 0 {
                50 * self.combo as u32 * self.level
            } else {
                0
            };

            let total = line_points + combo_points;
            self.score += total;

            let mut temp_board = self.board;
            let mut new_board = [[EMPTY; BOARD_WIDTH]; BOARD_HEIGHT];
            let mut dest = BOARD_HEIGHT - 1;
            for src in (0..BOARD_HEIGHT).rev() {
                if full_rows.contains(&src) {
                    continue;
                }
                new_board[dest] = temp_board[src];
                dest = dest.wrapping_sub(1);
            }
            temp_board = new_board;

            let is_all_clear = temp_board.iter().all(|row| row.iter().all(|&c| c == EMPTY));
            let pc_bonus = if is_all_clear {
                let pc_base = match cleared {
                    1 => 800,
                    2 => 1200,
                    3 => 1800,
                    4 if is_difficult && self.back_to_back => 3200,
                    4 => 2000,
                    _ => 0,
                };
                pc_base * self.level
            } else {
                0
            };
            self.score += pc_bonus;

            let mut label = String::new();
            if is_tspin {
                if is_mini {
                    label.push_str("Mini T-Spin ");
                } else {
                    label.push_str("T-Spin ");
                }
            }
            if is_difficult && self.back_to_back {
                label.insert_str(0, "B2B ");
            }
            label.push_str(match cleared {
                1 => "Single",
                2 => "Double",
                3 => "Triple",
                4 => "Tetris",
                _ => "",
            });
            if is_all_clear {
                label.push_str(" ALL CLEAR");
            }
            if self.combo > 0 {
                label.push_str(&format!(" Combo x{}", self.combo));
            }

            self.last_action = Some(ClearAction {
                label,
                points: total + pc_bonus,
            });
            self.last_action_time = Instant::now();

            if is_difficult {
                self.back_to_back = true;
            } else {
                self.back_to_back = false;
            }

            if self.mode == GameMode::Marathon || self.mode == GameMode::Endless {
                let new_level = self.start_level + self.lines / 10;
                self.level = match self.level_cap {
                    Some(cap) if self.start_level > cap => self.start_level,
                    Some(cap) => new_level.min(cap),
                    None => new_level,
                };
            }

            if (self.mode == GameMode::Marathon && self.lines >= self.marathon_goal)
                || (self.mode == GameMode::Sprint && self.lines >= self.sprint_goal)
            {
                self.cleared = true;
            }

            if self.line_clear_anim_enabled {
                self.line_clear_anim = Some(LineClearAnimation::new(full_rows));
            } else {
                self.remove_rows(&full_rows);
                self.are_timer = Some(Instant::now());
            }
            return true;
        } else {
            self.combo = -1;

            if is_tspin {
                let base = if is_mini { 100 } else { 400 };
                let points = base * self.level;
                self.score += points;
                let label = if is_mini {
                    "Mini T-Spin"
                } else {
                    "T-Spin"
                };
                self.last_action = Some(ClearAction {
                    label: label.to_string(),
                    points,
                });
                self.last_action_time = Instant::now();
            }
        }

        self.are_timer = Some(Instant::now());
        false
    }

    pub fn finish_clear(&mut self) {
        if let Some(anim) = self.line_clear_anim.take() {
            self.remove_rows(&anim.rows);
        }
        self.are_timer = Some(Instant::now());
    }

    pub fn update_animation(&mut self) -> bool {
        if let Some(ref mut anim) = self.line_clear_anim {
            let elapsed = anim.started_at.elapsed().as_millis() as u64;
            let total = LINE_CLEAR_ANIM_DURATION.as_millis() as u64;
            let phase_len = total / 3;
            let phase = if elapsed < phase_len {
                0
            } else if elapsed < phase_len * 2 {
                1
            } else if elapsed < total {
                2
            } else {
                return false;
            };
            anim.phase = phase;
            true
        } else {
            false
        }
    }

    pub fn is_animating(&self) -> bool {
        self.line_clear_anim.is_some()
    }

    pub fn in_are(&self) -> bool {
        self.are_timer.is_some()
    }

    pub fn check_are(&mut self) -> bool {
        if let Some(start) = self.are_timer {
            if start.elapsed() >= ARE_DELAY {
                self.are_timer = None;
                if self.cleared {
                    self.game_over = true;
                    return true;
                }
                self.spawn_next();
                return true;
            }
        }
        false
    }

    pub fn ghost_row(&self) -> i32 {
        let mut ghost = self.current.clone();
        loop {
            ghost.row += 1;
            if !self.fits(&ghost) {
                return ghost.row - 1;
            }
        }
    }

    /// Guideline gravity: time_per_row = (0.8 - (level-1)*0.007)^(level-1). Capped at 20G.
    pub fn gravity(&self) -> f64 {
        let lvl = self.level as f64;
        let time_per_row = (0.8 - (lvl - 1.0) * 0.007).powf(lvl - 1.0);
        let g = 1.0 / (time_per_row * 60.0);
        g.min(20.0)
    }

    pub fn drop_interval(&self) -> Duration {
        let g = self.gravity();
        if g >= 1.0 {
            Duration::from_micros(16_667)
        } else {
            let lvl = self.level as f64;
            let time_per_row = (0.8 - (lvl - 1.0) * 0.007).powf(lvl - 1.0);
            Duration::from_secs_f64(time_per_row)
        }
    }

    pub fn update_elapsed(&mut self) {
        self.elapsed = self.game_start.elapsed();
    }

    pub fn reset_game_start(&mut self) {
        self.game_start = Instant::now() - self.elapsed;
    }

    pub fn time_remaining(&self) -> Option<Duration> {
        if self.mode == GameMode::Ultra {
            Some(Duration::from_secs(self.ultra_time as u64).saturating_sub(self.elapsed))
        } else {
            None
        }
    }
}
