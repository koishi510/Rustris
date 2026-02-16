use std::time::{Duration, Instant};

use crate::piece::*;

pub const LOCK_DELAY: Duration = Duration::from_millis(500);

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
    pub game_over: bool,
    pub last_move: LastMove,
    pub combo: i32,
    pub back_to_back: bool,
    pub last_action: Option<ClearAction>,
    pub last_action_time: Instant,
    pub lock_delay: Option<Instant>,
}

impl Game {
    pub fn new() -> Self {
        let mut bag = Bag::new();
        let current_kind = bag.next();
        let mut next_queue = Vec::with_capacity(NEXT_COUNT);
        for _ in 0..NEXT_COUNT {
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
            level: 1,
            game_over: false,
            last_move: LastMove::None,
            combo: -1,
            back_to_back: false,
            last_action: None,
            last_action_time: Instant::now(),
            lock_delay: None,
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

    // T-Spin detection: 3-corner rule
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

        // Both front corners occupied -> Full T-Spin, otherwise Mini
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

    fn clear_lines(&mut self) -> u32 {
        let mut cleared = 0u32;
        let mut new_board = [[EMPTY; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut dest = BOARD_HEIGHT - 1;

        for src in (0..BOARD_HEIGHT).rev() {
            if self.board[src].iter().all(|&c| c != EMPTY) {
                cleared += 1;
            } else {
                new_board[dest] = self.board[src];
                dest = dest.wrapping_sub(1);
            }
        }
        self.board = new_board;
        cleared
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
        self.lock_and_advance();
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
        if !self.move_piece(1, 0) {
            if self.lock_delay.is_none() {
                self.lock_delay = Some(Instant::now());
            }
        }
    }

    pub fn lock_and_advance(&mut self) {
        let (is_tspin, is_mini) = self.detect_tspin();

        self.lock_current();
        let cleared = self.clear_lines();

        if cleared > 0 {
            self.lines += cleared;
            self.combo += 1;

            // Difficult clear: Tetris or any T-Spin line clear
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

            // Back-to-back bonus: x1.5 for consecutive difficult clears
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

            // All Clear (Perfect Clear)
            let is_all_clear = self.board.iter().all(|row| row.iter().all(|&c| c == EMPTY));
            let pc_bonus = if is_all_clear {
                let pc_base = match cleared {
                    1 => 800,
                    2 => 1200,
                    3 => 1800,
                    4 => 2000,
                    _ => 0,
                };
                pc_base * self.level
            } else {
                0
            };
            self.score += pc_bonus;

            // Build action label
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

            self.level = self.lines / 10 + 1;
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

        self.spawn_next();
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

    pub fn drop_interval(&self) -> Duration {
        let ms = match self.level {
            1 => 800,
            2 => 720,
            3 => 630,
            4 => 550,
            5 => 470,
            6 => 380,
            7 => 300,
            8 => 220,
            9 => 140,
            _ => 80,
        };
        Duration::from_millis(ms)
    }
}
