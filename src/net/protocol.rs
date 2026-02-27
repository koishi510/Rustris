use serde::{Deserialize, Serialize};

use crate::game::piece::{BOARD_WIDTH, BUFFER_HEIGHT, VISIBLE_HEIGHT};
use crate::game::settings::VersusSettings;
use crate::game::Game;

pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GarbageAttack {
    pub lines: u32,
    pub hole_column: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BoardSnapshot {
    pub board: Vec<u8>,
    pub current_cells: Vec<(i32, i32)>,
    pub current_kind: usize,
    pub score: u32,
    pub lines: u32,
    pub pending_garbage: u32,
}

impl BoardSnapshot {
    pub fn from_game(game: &Game, pending_garbage: u32) -> Self {
        let mut board = Vec::with_capacity(BOARD_WIDTH * VISIBLE_HEIGHT);
        for row in BUFFER_HEIGHT..(BUFFER_HEIGHT + VISIBLE_HEIGHT) {
            for col in 0..BOARD_WIDTH {
                board.push(game.board[row][col]);
            }
        }

        let current_cells = if game.is_animating() || game.in_are() {
            vec![]
        } else {
            game.current
                .cells()
                .iter()
                .map(|&(r, c)| (r - BUFFER_HEIGHT as i32, c))
                .collect()
        };

        let current_kind = game.current.kind;

        Self {
            board,
            current_cells,
            current_kind,
            score: game.score,
            lines: game.lines,
            pending_garbage,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MatchOutcome {
    Win,
    Lose,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetMessage {
    Hello { version: u8 },
    LobbySettings(VersusSettings),
    Ready,
    Countdown(u8),
    GameStart,
    GarbageAttack(GarbageAttack),
    BoardState(BoardSnapshot),
    PlayerDead,
    MatchResult(MatchOutcome),
    RematchRequest,
    RematchAccept,
    Disconnect,
}
