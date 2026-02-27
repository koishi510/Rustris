use std::time::Instant;

use crate::game::piece::*;

use super::{ClearAction, Game, GameMode, LineClearAnimation};

impl Game {
    fn detect_tspin(&self) -> (bool, bool) {
        if self.current.kind != KIND_T || self.last_move != super::LastMove::Rotate {
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

            let is_all_clear = (0..BOARD_HEIGHT).all(|r| {
                full_rows.contains(&r) || self.board[r].iter().all(|&c| c == EMPTY)
            });
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
                cleared_lines: cleared,
                is_tspin,
                is_mini,
                is_back_to_back: is_difficult && self.back_to_back,
                combo: self.combo,
                is_all_clear,
            });
            self.last_action_time = Instant::now();

            self.back_to_back = is_difficult;

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
                    cleared_lines: 0,
                    is_tspin: true,
                    is_mini,
                    is_back_to_back: false,
                    combo: -1,
                    is_all_clear: false,
                });
                self.last_action_time = Instant::now();
            }
        }

        self.are_timer = Some(Instant::now());
        false
    }
}
