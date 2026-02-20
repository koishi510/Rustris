use crossterm::style::Color;
use rand::seq::SliceRandom;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const EMPTY: u8 = 0;
pub const NEXT_COUNT: usize = 6;
pub const KIND_O: usize = 1;
pub const KIND_T: usize = 2;

// SRS rotation states: 7 pieces x 4 rotations, each cell is [row, col] offset
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

// T-Spin detection: front/back corner offsets relative to T center
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

// SRS wall kick tables: [dc, dr] offsets
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

#[derive(Clone)]
pub struct Piece {
    pub kind: usize,
    pub rotation: u8,
    pub row: i32,
    pub col: i32,
}

impl Piece {
    pub fn new(kind: usize) -> Self {
        let row = if kind == KIND_O { -1 } else { 0 };
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
}

impl Bag {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn next(&mut self) -> usize {
        if self.queue.is_empty() {
            let mut bag = vec![0, 1, 2, 3, 4, 5, 6];
            bag.shuffle(&mut rand::thread_rng());
            self.queue = bag;
        }
        self.queue.pop().unwrap()
    }
}
