use std::time::{Duration, Instant};

use crate::game::piece::*;

use super::Game;

impl Game {
    fn refresh_lock_delay(&mut self) {
        if self.lock_delay.is_some() {
            if self.is_on_ground() {
                if self.move_reset.is_none_or(|limit| self.move_reset_count < limit) {
                    self.lock_delay = Some(Instant::now());
                    self.move_reset_count += 1;
                }
            } else {
                self.lock_delay = None;
            }
        }
    }

    pub fn move_piece(&mut self, dr: i32, dc: i32) -> bool {
        let mut moved = self.current.clone();
        moved.row += dr;
        moved.col += dc;
        if self.fits(&moved) {
            self.current = moved;
            self.last_move = super::LastMove::Move;
            self.refresh_lock_delay();
            true
        } else {
            false
        }
    }

    fn try_rotate(&mut self, new_rotation: u8) {
        if self.current.kind == 1 {
            return;
        }

        if self.srs {
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
                    self.last_move = super::LastMove::Rotate;
                    self.refresh_lock_delay();
                    return;
                }
            }
        } else {
            let mut test = self.current.clone();
            test.rotation = new_rotation;
            if self.fits(&test) {
                self.current = test;
                self.last_move = super::LastMove::Rotate;
                self.refresh_lock_delay();
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
        } else if !self.move_piece(1, 0)
            && self.lock_delay.is_none()
        {
            self.lock_delay = Some(Instant::now());
        }
    }

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

    pub fn lock_delay_duration(&self) -> Duration {
        Duration::from_millis(self.lock_delay_ms as u64)
    }
}
