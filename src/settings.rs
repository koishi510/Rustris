use crate::piece::MAX_NEXT_COUNT;

#[derive(Clone, Copy)]
pub struct Settings {
    pub level: u32,                // 1..=20
    pub marathon_goal: u32,         // 10..=300, step 10
    pub sprint_goal: u32,          // 10..=100, step 10
    pub ultra_time: u32,           // 30..=300, step 10 (seconds)
    pub level_cap: Option<u32>,    // Some(1..=20) or None (no cap)
    pub ghost: bool,
    pub line_clear_anim: bool,
    pub next_count: usize,         // 1..=MAX_NEXT_COUNT
    pub bag_randomizer: bool,      // true=7-bag, false=pure random
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
        }
    }
}
