#[cfg(test)]
mod tests {
    use crate::game::*;
    use crate::game::garbage::*;
    use crate::game::piece::*;
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

    // --- Piece tests ---

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

    // --- Garbage queue tests ---

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
