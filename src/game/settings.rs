use serde::{Deserialize, Serialize};

use crate::game::piece::MAX_NEXT_COUNT;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub level: u32,
    pub marathon_goal: u32,
    pub sprint_goal: u32,
    pub ultra_time: u32,
    pub level_cap: Option<u32>,
    pub ghost: bool,
    pub line_clear_anim: bool,
    pub next_count: usize,
    pub bag_randomizer: bool,
    pub srs_enabled: bool,
    pub hold_enabled: bool,
    pub lock_delay_ms: u32,
    pub move_reset: Option<u32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            level: 1,
            marathon_goal: 150,
            sprint_goal: 40,
            ultra_time: 120,
            level_cap: Some(15),
            ghost: true,
            line_clear_anim: true,
            next_count: MAX_NEXT_COUNT,
            bag_randomizer: true,
            srs_enabled: true,
            hold_enabled: true,
            lock_delay_ms: 500,
            move_reset: Some(15),
        }
    }
}
