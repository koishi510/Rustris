use crate::game::ClearAction;

pub struct GarbageEvent {
    pub lines: u32,
    pub hole_column: usize,
}

pub struct GarbageQueue {
    pub pending: Vec<GarbageEvent>,
}

impl GarbageQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn total_pending(&self) -> u32 {
        self.pending.iter().map(|e| e.lines).sum()
    }

    pub fn push(&mut self, event: GarbageEvent) {
        self.pending.push(event);
    }

    pub fn cancel(&mut self, mut attack: u32) -> u32 {
        self.pending.retain_mut(|event| {
            if attack == 0 {
                return true;
            }
            if attack >= event.lines {
                attack -= event.lines;
                false
            } else {
                event.lines -= attack;
                attack = 0;
                true
            }
        });
        attack
    }

    pub fn drain_all(&mut self) -> Vec<GarbageEvent> {
        std::mem::take(&mut self.pending)
    }
}

fn combo_bonus(combo: i32) -> u32 {
    match combo {
        0..=1 => 0,
        2..=3 => 1,
        4..=5 => 2,
        6..=7 => 3,
        8..=10 => 4,
        _ => 5,
    }
}

pub fn calculate_attack(action: &ClearAction) -> u32 {
    if action.cleared_lines == 0 {
        return 0;
    }

    if action.is_all_clear {
        return 10;
    }

    let base = if action.is_tspin {
        if action.is_mini {
            match action.cleared_lines {
                1 => 0,
                2 => 1,
                _ => 0,
            }
        } else {
            match action.cleared_lines {
                1 => 2,
                2 => 4,
                3 => 6,
                _ => 0,
            }
        }
    } else {
        match action.cleared_lines {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 4,
            _ => 0,
        }
    };

    let b2b = if action.is_back_to_back { 1 } else { 0 };
    let combo = combo_bonus(action.combo);

    base + b2b + combo
}
