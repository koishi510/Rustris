use crossterm::style::Color;
use rand::seq::SliceRandom;
use rand::Rng;

pub const BOARD_WIDTH: usize = 10;
pub const VISIBLE_HEIGHT: usize = 20;
pub const BOARD_HEIGHT: usize = 40;
pub const BUFFER_HEIGHT: usize = BOARD_HEIGHT - VISIBLE_HEIGHT;
pub const EMPTY: u8 = 0;
pub const GARBAGE_CELL: u8 = 8;
pub const MAX_NEXT_COUNT: usize = 6;
pub const KIND_O: usize = 1;
pub const KIND_T: usize = 2;

pub const PIECE_STATES: [[[[i32; 2]; 4]; 4]; 7] = [
    // I
    [
        [[0, -1], [0, 0], [0, 1], [0, 2]],
        [[-1, 1], [0, 1], [1, 1], [2, 1]],
        [[1, -1], [1, 0], [1, 1], [1, 2]],
        [[-1, 0], [0, 0], [1, 0], [2, 0]],
    ],
    // O
    [
        [[0, 0], [0, 1], [1, 0], [1, 1]],
        [[0, 0], [0, 1], [1, 0], [1, 1]],
        [[0, 0], [0, 1], [1, 0], [1, 1]],
        [[0, 0], [0, 1], [1, 0], [1, 1]],
    ],
    // T
    [
        [[-1, 0], [0, -1], [0, 0], [0, 1]],
        [[-1, 0], [0, 0], [0, 1], [1, 0]],
        [[0, -1], [0, 0], [0, 1], [1, 0]],
        [[-1, 0], [0, -1], [0, 0], [1, 0]],
    ],
    // S
    [
        [[-1, 0], [-1, 1], [0, -1], [0, 0]],
        [[-1, 0], [0, 0], [0, 1], [1, 1]],
        [[0, 0], [0, 1], [1, -1], [1, 0]],
        [[-1, -1], [0, -1], [0, 0], [1, 0]],
    ],
    // Z
    [
        [[-1, -1], [-1, 0], [0, 0], [0, 1]],
        [[-1, 1], [0, 0], [0, 1], [1, 0]],
        [[0, -1], [0, 0], [1, 0], [1, 1]],
        [[-1, 0], [0, -1], [0, 0], [1, -1]],
    ],
    // L
    [
        [[-1, 1], [0, -1], [0, 0], [0, 1]],
        [[-1, 0], [0, 0], [1, 0], [1, 1]],
        [[0, -1], [0, 0], [0, 1], [1, -1]],
        [[-1, -1], [-1, 0], [0, 0], [1, 0]],
    ],
    // J
    [
        [[-1, -1], [0, -1], [0, 0], [0, 1]],
        [[-1, 0], [-1, 1], [0, 0], [1, 0]],
        [[0, -1], [0, 0], [0, 1], [1, 1]],
        [[-1, 0], [0, 0], [1, -1], [1, 0]],
    ],
];

pub const T_FRONT_CORNERS: [[[i32; 2]; 2]; 4] = [
    [[-1, -1], [-1, 1]],  // 0
    [[-1, 1], [1, 1]],    // R
    [[1, -1], [1, 1]],    // 2
    [[-1, -1], [1, -1]],  // L
];
pub const T_BACK_CORNERS: [[[i32; 2]; 2]; 4] = [
    [[1, -1], [1, 1]],    // 0
    [[-1, -1], [1, -1]],  // R
    [[-1, -1], [-1, 1]],  // 2
    [[-1, 1], [1, 1]],    // L
];

pub const KICK_JLTSZ: [[[i32; 2]; 5]; 8] = [
    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
];

pub const KICK_I: [[[i32; 2]; 5]; 8] = [
    [[0, 0], [-2, 0], [1, 0], [-2, 1], [1, -2]],
    [[0, 0], [2, 0], [-1, 0], [2, -1], [-1, 2]],
    [[0, 0], [-1, 0], [2, 0], [-1, -2], [2, 1]],
    [[0, 0], [1, 0], [-2, 0], [1, 2], [-2, -1]],
    [[0, 0], [2, 0], [-1, 0], [2, -1], [-1, 2]],
    [[0, 0], [-2, 0], [1, 0], [-2, 1], [1, -2]],
    [[0, 0], [1, 0], [-2, 0], [1, 2], [-2, -1]],
    [[0, 0], [-1, 0], [2, 0], [-1, -2], [2, 1]],
];

pub fn kick_index(from: u8, to: u8) -> usize {
    match (from, to) {
        (0, 1) => 0,
        (1, 0) => 1,
        (1, 2) => 2,
        (2, 1) => 3,
        (2, 3) => 4,
        (3, 2) => 5,
        (3, 0) => 6,
        (0, 3) => 7,
        _ => 0,
    }
}

pub const PIECE_COLORS: [Color; 7] = [
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
    Color::Green,
    Color::Red,
    Color::DarkYellow,
    Color::Blue,
];

#[derive(Clone, Copy)]
pub struct Piece {
    pub kind: usize,
    pub rotation: u8,
    pub row: i32,
    pub col: i32,
}

impl Piece {
    pub fn new(kind: usize) -> Self {
        let row = if kind == KIND_O {
            BUFFER_HEIGHT as i32 - 1
        } else {
            BUFFER_HEIGHT as i32
        };
        Self {
            kind,
            rotation: 0,
            row,
            col: (BOARD_WIDTH as i32) / 2 - 1,
        }
    }

    pub fn blocks(&self) -> &[[i32; 2]; 4] {
        &PIECE_STATES[self.kind][self.rotation as usize]
    }

    pub fn cells(&self) -> [(i32, i32); 4] {
        let mut out = [(0i32, 0i32); 4];
        for (i, b) in self.blocks().iter().enumerate() {
            out[i] = (self.row + b[0], self.col + b[1]);
        }
        out
    }
}

pub struct Bag {
    queue: Vec<usize>,
    use_bag: bool,
}

impl Bag {
    pub fn new(use_bag: bool) -> Self {
        Self { queue: Vec::new(), use_bag }
    }

    pub fn next(&mut self) -> usize {
        if !self.use_bag {
            return rand::thread_rng().gen_range(0..7);
        }
        if self.queue.is_empty() {
            let mut bag = vec![0, 1, 2, 3, 4, 5, 6];
            bag.shuffle(&mut rand::thread_rng());
            self.queue = bag;
        }
        self.queue.pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kick_index_all_transitions() {
        assert_eq!(kick_index(0, 1), 0);
        assert_eq!(kick_index(1, 0), 1);
        assert_eq!(kick_index(1, 2), 2);
        assert_eq!(kick_index(2, 1), 3);
        assert_eq!(kick_index(2, 3), 4);
        assert_eq!(kick_index(3, 2), 5);
        assert_eq!(kick_index(3, 0), 6);
        assert_eq!(kick_index(0, 3), 7);
    }

    #[test]
    fn piece_new_i_spawn() {
        let p = Piece::new(0);
        assert_eq!(p.kind, 0);
        assert_eq!(p.rotation, 0);
        assert_eq!(p.row, (BOARD_HEIGHT - VISIBLE_HEIGHT) as i32);
        assert_eq!(p.col, (BOARD_WIDTH as i32) / 2 - 1);
    }

    #[test]
    fn piece_new_o_spawn() {
        let p = Piece::new(KIND_O);
        assert_eq!(p.row, (BOARD_HEIGHT - VISIBLE_HEIGHT) as i32 - 1);
        assert_eq!(p.col, (BOARD_WIDTH as i32) / 2 - 1);
    }

    #[test]
    fn piece_new_t_spawn() {
        let p = Piece::new(KIND_T);
        assert_eq!(p.row, (BOARD_HEIGHT - VISIBLE_HEIGHT) as i32);
        assert_eq!(p.col, (BOARD_WIDTH as i32) / 2 - 1);
    }

    #[test]
    fn bag_7bag_completeness() {
        let mut bag = Bag::new(true);
        let mut counts = [0u32; 7];
        for _ in 0..7 {
            let kind = bag.next();
            assert!(kind < 7);
            counts[kind] += 1;
        }
        for count in counts {
            assert_eq!(count, 1);
        }
    }

    #[test]
    fn bag_7bag_two_cycles() {
        let mut bag = Bag::new(true);
        let mut counts = [0u32; 7];
        for _ in 0..14 {
            let kind = bag.next();
            counts[kind] += 1;
        }
        for count in counts {
            assert_eq!(count, 2);
        }
    }

    #[test]
    fn bag_random_mode() {
        let mut bag = Bag::new(false);
        for _ in 0..100 {
            let kind = bag.next();
            assert!(kind < 7);
        }
    }
}
