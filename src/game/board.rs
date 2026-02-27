use std::time::{Duration, Instant};

use crate::game::piece::*;
use crate::game::settings::Settings;

use super::{Game, GameMode, LastMove};

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
            srs_enabled: settings.srs_enabled,
            hold_enabled: settings.hold_enabled,
            lock_delay_ms: settings.lock_delay_ms,
            move_reset: settings.move_reset,
            move_reset_count: 0,
            garbage_rise_anim: None,
        }
    }

    pub(super) fn pop_next(&mut self) -> Piece {
        if self.next_queue.is_empty() {
            return Piece::new(self.bag.next());
        }
        let kind = self.next_queue.remove(0);
        self.next_queue.push(self.bag.next());
        Piece::new(kind)
    }

    pub(super) fn is_occupied(&self, r: i32, c: i32) -> bool {
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

    pub(super) fn find_full_rows(&self) -> Vec<usize> {
        let mut rows = Vec::new();
        for r in 0..BOARD_HEIGHT {
            if self.board[r].iter().all(|&c| c != EMPTY) {
                rows.push(r);
            }
        }
        rows
    }

    pub(super) fn remove_rows(&mut self, rows: &[usize]) {
        let mut new_board = [[EMPTY; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut dest = BOARD_HEIGHT - 1;
        for src in (0..BOARD_HEIGHT).rev() {
            if rows.contains(&src) {
                continue;
            }
            new_board[dest] = self.board[src];
            dest = dest.saturating_sub(1);
        }
        self.board = new_board;
    }

    pub(super) fn spawn_next(&mut self) {
        self.current = self.pop_next();
        self.hold_used = false;
        self.last_move = LastMove::None;
        self.lock_delay = None;
        self.move_reset_count = 0;
        if !self.fits(&self.current) {
            self.game_over = true;
        }
    }

    pub(super) fn is_on_ground(&self) -> bool {
        let mut test = self.current;
        test.row += 1;
        !self.fits(&test)
    }

    pub fn hold_piece(&mut self) {
        if !self.hold_enabled || self.hold_used {
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
        self.move_reset_count = 0;
    }

    pub fn ghost_row(&self) -> i32 {
        let mut ghost = self.current;
        loop {
            ghost.row += 1;
            if !self.fits(&ghost) {
                return ghost.row - 1;
            }
        }
    }

    pub fn has_blocks_in_buffer(&self) -> bool {
        for r in 0..BUFFER_HEIGHT {
            for c in 0..BOARD_WIDTH {
                if self.board[r][c] != EMPTY {
                    return true;
                }
            }
        }
        false
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
