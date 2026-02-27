use std::time::Instant;

use crate::game::garbage::GarbageEvent;
use crate::game::piece::*;

use super::{Game, GarbageRiseAnimation, ARE_DELAY, LINE_CLEAR_ANIM_DURATION};

const GARBAGE_RISE_INTERVAL_MS: u64 = 40;

impl Game {
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

    pub fn receive_garbage(&mut self, lines: u32, hole_column: usize) {
        let lines = lines as usize;
        if lines == 0 {
            return;
        }
        for r in 0..BOARD_HEIGHT.saturating_sub(lines) {
            self.board[r] = self.board[r + lines];
        }
        for r in BOARD_HEIGHT.saturating_sub(lines)..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                self.board[r][c] = if c == hole_column { EMPTY } else { GARBAGE_CELL };
            }
        }
    }

    pub fn begin_garbage_rise(&mut self, events: Vec<GarbageEvent>) {
        if events.is_empty() {
            return;
        }
        let anim_events: Vec<(u32, usize)> = events
            .into_iter()
            .map(|e| (e.lines, e.hole_column))
            .collect();
        self.garbage_rise_anim = Some(GarbageRiseAnimation {
            events: anim_events,
            started_at: Instant::now(),
            lines_applied: 0,
        });
    }

    pub fn is_garbage_animating(&self) -> bool {
        self.garbage_rise_anim.is_some()
    }

    pub fn update_garbage_animation(&mut self) -> bool {
        let anim = match self.garbage_rise_anim.as_ref() {
            Some(a) => a,
            None => return false,
        };

        let elapsed_ms = anim.started_at.elapsed().as_millis() as u64;
        let target_lines = (elapsed_ms / GARBAGE_RISE_INTERVAL_MS) as u32;

        let mut total_lines: u32 = 0;
        for (lines, _) in &anim.events {
            total_lines += lines;
        }

        let lines_to_apply = target_lines.min(total_lines);
        let already_applied = anim.lines_applied;

        if lines_to_apply <= already_applied {
            return lines_to_apply < total_lines;
        }

        let mut applied_so_far: u32 = 0;
        let events_snapshot: Vec<(u32, usize)> = anim.events.clone();

        for (event_lines, hole) in &events_snapshot {
            let event_start = applied_so_far;
            let event_end = applied_so_far + event_lines;
            applied_so_far = event_end;

            if event_end <= already_applied {
                continue;
            }

            let start_in_event = already_applied.saturating_sub(event_start);
            let end_in_event = (lines_to_apply - event_start).min(*event_lines);

            for _ in start_in_event..end_in_event {
                self.receive_garbage(1, *hole);
            }

            if applied_so_far >= lines_to_apply {
                break;
            }
        }

        if let Some(anim) = self.garbage_rise_anim.as_mut() {
            anim.lines_applied = lines_to_apply;
        }

        if lines_to_apply >= total_lines {
            self.garbage_rise_anim = None;
            return false;
        }

        true
    }
}
