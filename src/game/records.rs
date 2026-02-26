use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::game::GameMode;

#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreRecord {
    pub score: u32,
    pub lines: u32,
    pub level: u32,
    pub time: Option<u64>,
    pub date: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Records {
    pub marathon: Vec<ScoreRecord>,
    pub sprint: Vec<ScoreRecord>,
    pub ultra: Vec<ScoreRecord>,
    #[serde(default)]
    pub endless: Vec<ScoreRecord>,
}

fn records_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("rustris");
    path.push("records.json");
    path
}

impl Records {
    pub fn load() -> Self {
        let path = records_path();
        match fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = records_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, data);
        }
    }

    pub fn add(&mut self, mode: GameMode, record: ScoreRecord) -> Option<usize> {
        let list = match mode {
            GameMode::Marathon => &mut self.marathon,
            GameMode::Sprint => &mut self.sprint,
            GameMode::Ultra => &mut self.ultra,
            GameMode::Endless => &mut self.endless,
            GameMode::Versus => return None,
        };

        match mode {
            GameMode::Sprint => {
                let time = record.time?;
                let pos = list
                    .iter()
                    .position(|r| r.time.is_none_or(|t| time < t))
                    .unwrap_or(list.len());
                if pos >= 10 {
                    return None;
                }
                list.insert(pos, record);
                list.truncate(10);
                Some(pos)
            }
            GameMode::Marathon | GameMode::Ultra | GameMode::Endless | GameMode::Versus => {
                let score = record.score;
                let pos = list
                    .iter()
                    .position(|r| score > r.score)
                    .unwrap_or(list.len());
                if pos >= 10 {
                    return None;
                }
                list.insert(pos, record);
                list.truncate(10);
                Some(pos)
            }
        }
    }
}
