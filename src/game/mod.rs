mod animation;
mod movement;
mod scoring;

pub mod garbage;
pub mod piece;
pub mod records;
pub mod settings;
pub mod types;
pub use types::*;

use std::time::{Duration, Instant};

use crate::game::piece::*;
use crate::game::settings::Settings;

pub struct Game {
    pub board: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current: Piece,
    pub next_queue: Vec<usize>,
    pub hold: Option<usize>,
    pub hold_used: bool,
    bag: Bag,
    pub score: u32,
    pub lines: u32,
    pub level: u32,
    pub start_level: u32,
    pub game_over: bool,
    pub last_move: LastMove,
    pub combo: i32,
    pub back_to_back: bool,
    pub last_action: Option<ClearAction>,
    pub last_action_time: Instant,
    pub lock_delay: Option<Instant>,
    pub line_clear_anim: Option<LineClearAnimation>,
    pub are_timer: Option<Instant>,
    pub mode: GameMode,
    pub game_start: Instant,
    pub elapsed: Duration,
    pub cleared: bool,
    pub marathon_goal: u32,
    pub sprint_goal: u32,
    pub ultra_time: u32,
    pub level_cap: Option<u32>,
    pub ghost_enabled: bool,
    pub line_clear_anim_enabled: bool,
    pub next_count: usize,
    pub srs: bool,
    pub hold_enabled: bool,
    pub lock_delay_ms: u32,
    pub move_reset: Option<u32>,
    pub move_reset_count: u32,
    pub garbage_rise_anim: Option<GarbageRiseAnimation>,
}

impl Game {
    pub fn new(mode: GameMode, settings: &Settings) -> Self {
        let start_level = match mode {
            GameMode::Marathon | GameMode::Endless => settings.level,
            GameMode::Sprint | GameMode::Ultra => 1,
            GameMode::Versus => settings.level,
        };
        let mut bag = Bag::new(settings.bag_randomizer);
        let current_kind = bag.next();
        let mut next_queue = Vec::with_capacity(settings.next_count);
        for _ in 0..settings.next_count {
            next_queue.push(bag.next());
        }
        Self {
            board: [[EMPTY; BOARD_WIDTH]; BOARD_HEIGHT],
            current: Piece::new(current_kind),
            next_queue,
            hold: None,
            hold_used: false,
            bag,
            score: 0,
            lines: 0,
            level: start_level,
            start_level,
            game_over: false,
            last_move: LastMove::None,
            combo: -1,
            back_to_back: false,
            last_action: None,
            last_action_time: Instant::now(),
            lock_delay: None,
            line_clear_anim: None,
            are_timer: None,
            mode,
            game_start: Instant::now(),
            elapsed: Duration::ZERO,
            cleared: false,
            marathon_goal: settings.marathon_goal,
            sprint_goal: settings.sprint_goal,
            ultra_time: settings.ultra_time,
            level_cap: settings.level_cap,
            ghost_enabled: settings.ghost,
            line_clear_anim_enabled: settings.line_clear_anim,
            next_count: settings.next_count,
            srs: settings.srs,
            hold_enabled: settings.hold_enabled,
            lock_delay_ms: settings.lock_delay_ms,
            move_reset: settings.move_reset,
            move_reset_count: 0,
            garbage_rise_anim: None,
        }
    }

    fn pop_next(&mut self) -> Piece {
        if self.next_queue.is_empty() {
            return Piece::new(self.bag.next());
        }
        let kind = self.next_queue.remove(0);
        self.next_queue.push(self.bag.next());
        Piece::new(kind)
    }

    fn is_occupied(&self, r: i32, c: i32) -> bool {
        if r < 0 || r >= BOARD_HEIGHT as i32 || c < 0 || c >= BOARD_WIDTH as i32 {
            return true;
        }
        self.board[r as usize][c as usize] != EMPTY
    }

    pub fn fits(&self, piece: &Piece) -> bool {
        for (r, c) in piece.cells() {
            if c < 0 || c >= BOARD_WIDTH as i32 || r >= BOARD_HEIGHT as i32 {
                return false;
            }
            if r >= 0 && self.board[r as usize][c as usize] != EMPTY {
                return false;
            }
        }
        true
    }

    fn find_full_rows(&self) -> Vec<usize> {
        let mut rows = Vec::new();
        for r in 0..BOARD_HEIGHT {
            if self.board[r].iter().all(|&c| c != EMPTY) {
                rows.push(r);
            }
        }
        rows
    }

    fn remove_rows(&mut self, rows: &[usize]) {
        let mut new_board = [[EMPTY; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut dest = BOARD_HEIGHT - 1;
        for src in (0..BOARD_HEIGHT).rev() {
            if rows.contains(&src) {
                continue;
            }
            new_board[dest] = self.board[src];
            dest = dest.saturating_sub(1);
        }
        self.board = new_board;
    }

    fn spawn_next(&mut self) {
        self.current = self.pop_next();
        self.hold_used = false;
        self.last_move = LastMove::None;
        self.lock_delay = None;
        self.move_reset_count = 0;
        if !self.fits(&self.current) {
            self.game_over = true;
        }
    }

    fn is_on_ground(&self) -> bool {
        let mut test = self.current.clone();
        test.row += 1;
        !self.fits(&test)
    }

    pub fn hold_piece(&mut self) {
        if !self.hold_enabled || self.hold_used {
            return;
        }
        self.hold_used = true;
        self.lock_delay = None;
        let cur_kind = self.current.kind;
        match self.hold {
            Some(held_kind) => {
                self.hold = Some(cur_kind);
                self.current = Piece::new(held_kind);
                if !self.fits(&self.current) {
                    self.game_over = true;
                }
            }
            None => {
                self.hold = Some(cur_kind);
                self.current = self.pop_next();
                if !self.fits(&self.current) {
                    self.game_over = true;
                }
            }
        }
        self.last_move = LastMove::None;
        self.move_reset_count = 0;
    }

    pub fn ghost_row(&self) -> i32 {
        let mut ghost = self.current.clone();
        loop {
            ghost.row += 1;
            if !self.fits(&ghost) {
                return ghost.row - 1;
            }
        }
    }

    pub fn update_elapsed(&mut self) {
        self.elapsed = self.game_start.elapsed();
    }

    pub fn reset_game_start(&mut self) {
        self.game_start = Instant::now() - self.elapsed;
    }

    pub fn time_remaining(&self) -> Option<Duration> {
        if self.mode == GameMode::Ultra {
            Some(Duration::from_secs(self.ultra_time as u64).saturating_sub(self.elapsed))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::settings::Settings;

    fn test_settings() -> Settings {
        Settings {
            line_clear_anim: false,
            ..Settings::default()
        }
    }

    fn make_game() -> Game {
        Game::new(GameMode::Versus, &test_settings())
    }

    #[test]
    fn fits_empty_board() {
        let game = make_game();
        let piece = Piece::new(0);
        assert!(game.fits(&piece));
    }

    #[test]
    fn fits_occupied_cell() {
        let mut game = make_game();
        let p = Piece::new(KIND_T);
        let cells = p.cells();
        let (r, c) = cells.iter().find(|&&(r, _)| r >= 0).unwrap();
        game.board[*r as usize][*c as usize] = 1;
        assert!(!game.fits(&p));
    }

    #[test]
    fn fits_out_of_bounds_left() {
        let game = make_game();
        let mut piece = Piece::new(0);
        piece.col = -5;
        assert!(!game.fits(&piece));
    }

    #[test]
    fn fits_out_of_bounds_right() {
        let game = make_game();
        let mut piece = Piece::new(0);
        piece.col = BOARD_WIDTH as i32;
        assert!(!game.fits(&piece));
    }

    #[test]
    fn fits_out_of_bounds_bottom() {
        let game = make_game();
        let mut piece = Piece::new(0);
        piece.row = BOARD_HEIGHT as i32;
        assert!(!game.fits(&piece));
    }

    #[test]
    fn receive_garbage_bottom_rows() {
        let mut game = make_game();
        game.receive_garbage(2, 3);
        for r in (BOARD_HEIGHT - 2)..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                if c == 3 {
                    assert_eq!(game.board[r][c], EMPTY);
                } else {
                    assert_eq!(game.board[r][c], GARBAGE_CELL);
                }
            }
        }
        for c in 0..BOARD_WIDTH {
            assert_eq!(game.board[0][c], EMPTY);
        }
    }

    #[test]
    fn receive_garbage_shifts_up() {
        let mut game = make_game();
        game.board[BOARD_HEIGHT - 1][0] = 5;
        game.receive_garbage(1, 0);
        assert_eq!(game.board[BOARD_HEIGHT - 2][0], 5);
        assert_eq!(game.board[BOARD_HEIGHT - 1][0], EMPTY);
        assert_eq!(game.board[BOARD_HEIGHT - 1][1], GARBAGE_CELL);
    }

    #[test]
    fn receive_garbage_zero_noop() {
        let mut game = make_game();
        let board_before = game.board;
        game.receive_garbage(0, 0);
        assert_eq!(game.board, board_before);
    }

    fn setup_full_rows(game: &mut Game, count: usize) {
        for r in (BOARD_HEIGHT - count)..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                game.board[r][c] = 1;
            }
        }
    }

    #[test]
    fn scoring_single() {
        let mut game = make_game();
        setup_full_rows(&mut game, 1);
        game.current = Piece::new(0);
        game.current.row = (BOARD_HEIGHT as i32) - 2;
        game.current.col = 0;
        game.lock_and_begin_clear();
        assert!(game.score >= 100);
    }

    #[test]
    fn scoring_tetris() {
        let mut game = make_game();
        setup_full_rows(&mut game, 4);
        game.current = Piece::new(0);
        game.current.row = (BOARD_HEIGHT as i32) - 5;
        game.lock_and_begin_clear();
        assert!(game.score >= 800);
        assert_eq!(game.lines, 4);
    }

    #[test]
    fn scoring_b2b_tetris() {
        let mut game = make_game();
        game.back_to_back = true;
        setup_full_rows(&mut game, 4);
        game.current = Piece::new(0);
        game.current.row = (BOARD_HEIGHT as i32) - 5;
        game.lock_and_begin_clear();
        assert!(game.score >= 1200);
    }

    #[test]
    fn scoring_combo() {
        let mut game = make_game();
        game.combo = 1;
        setup_full_rows(&mut game, 1);
        game.current = Piece::new(0);
        game.current.row = (BOARD_HEIGHT as i32) - 2;
        game.lock_and_begin_clear();
        assert!(game.score >= 200);
        assert_eq!(game.combo, 2);
    }

    #[test]
    fn scoring_tspin_detection() {
        let mut game = make_game();

        let t_row = (BOARD_HEIGHT as i32) - 1;
        let t_col = 4;
        game.current = Piece::new(KIND_T);
        game.current.row = t_row;
        game.current.col = t_col;
        game.current.rotation = 0;
        game.last_move = LastMove::Rotate;

        // Back corners are out of bounds (below board), front-left occupied -> 3 corners
        let fr = (t_row - 1) as usize;
        game.board[fr][t_col as usize - 1] = 1;

        for c in 0..BOARD_WIDTH {
            if c != (t_col - 1) as usize && c != t_col as usize && c != (t_col + 1) as usize {
                game.board[t_row as usize][c] = 1;
            }
        }

        game.lock_and_begin_clear();
        if let Some(action) = &game.last_action {
            assert!(action.is_tspin);
        }
    }

    #[test]
    fn no_clear_resets_combo() {
        let mut game = make_game();
        game.combo = 5;
        game.current = Piece::new(0);
        game.current.row = 0;
        game.lock_and_begin_clear();
        assert_eq!(game.combo, -1);
    }
}
