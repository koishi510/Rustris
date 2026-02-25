use crossterm::{cursor, execute, style::{Color, Stylize}};
use std::io::{self, Write};

use crate::game::Game;
use crate::piece::*;

use super::{color_for, draw_piece_preview, draw_right_panel, draw_title, left_panel_pad, LEFT_W};

pub fn draw(stdout: &mut io::Stdout, game: &Game) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let animating = game.is_animating();
    let anim_rows: Vec<usize> = game
        .line_clear_anim
        .as_ref()
        .map(|a| a.rows.clone())
        .unwrap_or_default();
    let anim_phase = game
        .line_clear_anim
        .as_ref()
        .map(|a| a.phase)
        .unwrap_or(0);

    let ghost_cells: [(i32, i32); 4];
    let current_cells: [(i32, i32); 4];
    let current_color: Color;
    if animating || game.in_are() {
        ghost_cells = [(-1, -1); 4];
        current_cells = [(-1, -1); 4];
        current_color = Color::White;
    } else {
        if game.ghost_enabled {
            let ghost_row = game.ghost_row();
            ghost_cells = {
                let mut g = game.current.clone();
                g.row = ghost_row;
                g.cells()
            };
        } else {
            ghost_cells = [(-1, -1); 4];
        }
        current_cells = game.current.cells();
        current_color = PIECE_COLORS[game.current.kind];
    }

    write!(stdout, "{:LEFT_W$}╔", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╗\x1b[K\r\n")?;

    for row in 0..BOARD_HEIGHT {
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
            if anim_rows.contains(&row) {
                match anim_phase {
                    0 => write!(stdout, "{}", "██".with(Color::White))?,
                    1 => write!(stdout, "{}", "▓▓".with(Color::DarkGrey))?,
                    _ => write!(stdout, "  ")?,
                }
            } else if current_cells.contains(&(row as i32, col as i32)) {
                write!(stdout, "{}", "██".with(current_color))?;
            } else if ghost_cells.contains(&(row as i32, col as i32))
                && game.board[row][col] == EMPTY
            {
                write!(stdout, "{}", "░░".with(current_color))?;
            } else {
                let id = game.board[row][col];
                if id == EMPTY {
                    write!(stdout, "  ")?;
                } else {
                    write!(stdout, "{}", "██".with(color_for(id)))?;
                }
            }
        }

        write!(stdout, "║")?;
        draw_right_panel(stdout, game, row)?;
        write!(stdout, "\x1b[K\r\n")?;
    }

    write!(stdout, "{:LEFT_W$}╚", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╝\x1b[K\r\n")?;

    write!(stdout, "\x1b[J")?;

    stdout.flush()?;
    Ok(())
}
