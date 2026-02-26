use serde::{Deserialize, Serialize};

use crate::game::settings::VersusSettings;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MatchOutcome {
    Win,
    Lose,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetMessage {
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
