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
        while attack > 0 && !self.pending.is_empty() {
            let front = &mut self.pending[0];
            if attack >= front.lines {
                attack -= front.lines;
                self.pending.remove(0);
            } else {
                front.lines -= attack;
                attack = 0;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action(
        cleared_lines: u32,
        is_tspin: bool,
        is_mini: bool,
        is_back_to_back: bool,
        combo: i32,
        is_all_clear: bool,
    ) -> ClearAction {
        ClearAction {
            label: String::new(),
            points: 0,
            cleared_lines,
            is_tspin,
            is_mini,
            is_back_to_back,
            combo,
            is_all_clear,
        }
    }

    #[test]
    fn cancel_exact_match() {
        let mut q = GarbageQueue::new();
        q.push(GarbageEvent { lines: 3, hole_column: 0 });
        let remaining = q.cancel(3);
        assert_eq!(remaining, 0);
        assert!(q.pending.is_empty());
    }

    #[test]
    fn cancel_partial() {
        let mut q = GarbageQueue::new();
        q.push(GarbageEvent { lines: 5, hole_column: 0 });
        let remaining = q.cancel(3);
        assert_eq!(remaining, 0);
        assert_eq!(q.pending.len(), 1);
        assert_eq!(q.pending[0].lines, 2);
    }

    #[test]
    fn cancel_overflow() {
        let mut q = GarbageQueue::new();
        q.push(GarbageEvent { lines: 2, hole_column: 0 });
        let remaining = q.cancel(5);
        assert_eq!(remaining, 3);
        assert!(q.pending.is_empty());
    }

    #[test]
    fn cancel_multi_event() {
        let mut q = GarbageQueue::new();
        q.push(GarbageEvent { lines: 2, hole_column: 0 });
        q.push(GarbageEvent { lines: 3, hole_column: 1 });
        let remaining = q.cancel(4);
        assert_eq!(remaining, 0);
        assert_eq!(q.pending.len(), 1);
        assert_eq!(q.pending[0].lines, 1);
        assert_eq!(q.pending[0].hole_column, 1);
    }

    #[test]
    fn cancel_zero_attack() {
        let mut q = GarbageQueue::new();
        q.push(GarbageEvent { lines: 3, hole_column: 0 });
        let remaining = q.cancel(0);
        assert_eq!(remaining, 0);
        assert_eq!(q.total_pending(), 3);
    }

    #[test]
    fn attack_single() {
        let a = make_action(1, false, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 0);
    }

    #[test]
    fn attack_double() {
        let a = make_action(2, false, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 1);
    }

    #[test]
    fn attack_triple() {
        let a = make_action(3, false, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 2);
    }

    #[test]
    fn attack_tetris() {
        let a = make_action(4, false, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 4);
    }

    #[test]
    fn attack_tspin_single() {
        let a = make_action(1, true, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 2);
    }

    #[test]
    fn attack_tspin_double() {
        let a = make_action(2, true, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 4);
    }

    #[test]
    fn attack_tspin_triple() {
        let a = make_action(3, true, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 6);
    }

    #[test]
    fn attack_mini_tspin_single() {
        let a = make_action(1, true, true, false, 0, false);
        assert_eq!(calculate_attack(&a), 0);
    }

    #[test]
    fn attack_mini_tspin_double() {
        let a = make_action(2, true, true, false, 0, false);
        assert_eq!(calculate_attack(&a), 1);
    }

    #[test]
    fn attack_b2b_bonus() {
        let a = make_action(4, false, false, true, 0, false);
        assert_eq!(calculate_attack(&a), 5);
    }

    #[test]
    fn attack_combo_bonus() {
        let a = make_action(2, false, false, false, 2, false);
        assert_eq!(calculate_attack(&a), 2);
    }

    #[test]
    fn attack_all_clear() {
        let a = make_action(1, false, false, false, 0, true);
        assert_eq!(calculate_attack(&a), 10);
    }

    #[test]
    fn attack_zero_lines() {
        let a = make_action(0, false, false, false, 0, false);
        assert_eq!(calculate_attack(&a), 0);
    }
}
