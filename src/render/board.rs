use crossterm::{cursor, execute};
use std::io::{self, Write};

use crate::game::Game;
use crate::game::piece::*;

use super::{draw_board_bottom, draw_board_top, draw_piece_preview, draw_right_panel, draw_title, left_panel_pad, BoardRenderState, draw_board_cell, LEFT_W};

pub fn draw(stdout: &mut io::Stdout, game: &Game) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let state = BoardRenderState::from_game(game);

    draw_board_top(stdout)?;

    for row in 0..VISIBLE_HEIGHT {
        let board_row = row + BUFFER_HEIGHT;
        match row {
            0 if game.next_count > 0 => {
                write!(stdout, "{:<LEFT_W$}", "  NEXT:")?;
            }
            2..=19 if game.next_count > 0 => {
                let offset = row - 2;
                let slot = offset / 3;
                let in_slot = offset % 3;
                if slot < game.next_count && in_slot < 2 {
                    draw_piece_preview(
                        stdout,
                        game.next_queue[slot],
                        in_slot as i32,
                    )?;
                    left_panel_pad(stdout, 10)?;
                } else {
                    write!(stdout, "{:LEFT_W$}", "")?;
                }
            }
            _ => {
                write!(stdout, "{:LEFT_W$}", "")?;
            }
        }

        write!(stdout, "║")?;
        for col in 0..BOARD_WIDTH {
            draw_board_cell(stdout, &game.board, board_row, col, &state)?;
        }

        write!(stdout, "║")?;
        draw_right_panel(stdout, game, row)?;
        write!(stdout, "\x1b[K\r\n")?;
    }

    draw_board_bottom(stdout)?;

    write!(stdout, "\x1b[J")?;

    stdout.flush()?;
    Ok(())
}
