use crossterm::{cursor, execute, style::{Color, Stylize}};
use std::io::{self, Write};
use std::time::Duration;

use crate::game::{Game, GameMode};
use crate::piece::*;

use super::{color_for, draw_piece_preview, draw_title, left_panel_pad, LEFT_W};

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

    let show_action = game.last_action.is_some()
        && game.last_action_time.elapsed() < Duration::from_secs(3);

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
        match row {
            0 if game.hold_enabled => {
                if game.hold_used {
                    write!(stdout, "  {}", "HOLD:".with(Color::DarkGrey))?;
                } else {
                    write!(stdout, "  HOLD:")?;
                }
            }
            2 | 3 if game.hold_enabled => {
                let pr = (row - 2) as i32;
                if let Some(kind) = game.hold {
                    draw_piece_preview(stdout, kind, pr)?;
                }
            }
            5 => match game.mode {
                GameMode::Marathon | GameMode::Endless => write!(stdout, "  SCORE: {}", game.score)?,
                GameMode::Sprint => {
                    let secs = game.elapsed.as_secs();
                    let centis = game.elapsed.subsec_millis() / 10;
                    write!(stdout, "  TIME: {}:{:02}.{:02}", secs / 60, secs % 60, centis)?;
                }
                GameMode::Ultra => {
                    if let Some(rem) = game.time_remaining() {
                        let secs = rem.as_secs();
                        let centis = rem.subsec_millis() / 10;
                        write!(stdout, "  TIME: {}:{:02}.{:02}", secs / 60, secs % 60, centis)?;
                    }
                }
            },
            6 => match game.mode {
                GameMode::Marathon => write!(stdout, "  LINES: {} / {}", game.lines, game.marathon_goal)?,
                GameMode::Endless => write!(stdout, "  LINES: {}", game.lines)?,
                GameMode::Sprint => write!(stdout, "  LINES: {} / {}", game.lines, game.sprint_goal)?,
                GameMode::Ultra => write!(stdout, "  SCORE: {}", game.score)?,
            },
            7 => match game.mode {
                GameMode::Marathon | GameMode::Sprint | GameMode::Endless => write!(stdout, "  LEVEL: {}", game.level)?,
                GameMode::Ultra => write!(stdout, "  LINES: {}", game.lines)?,
            },
            8 => match game.mode {
                GameMode::Ultra => write!(stdout, "  LEVEL: {}", game.level)?,
                _ => {
                    if show_action {
                        let action = game.last_action.as_ref().unwrap();
                        write!(stdout, "  {}", action.label.as_str().with(Color::Yellow))?;
                    }
                }
            },
            9 => match game.mode {
                GameMode::Ultra => {
                    if show_action {
                        let action = game.last_action.as_ref().unwrap();
                        write!(stdout, "  {}", action.label.as_str().with(Color::Yellow))?;
                    }
                }
                _ => {
                    if show_action {
                        let action = game.last_action.as_ref().unwrap();
                        write!(stdout, "  {}", format!("+{}", action.points).with(Color::Yellow))?;
                    }
                }
            },
            10 => {
                if game.mode == GameMode::Ultra && show_action {
                    let action = game.last_action.as_ref().unwrap();
                    write!(stdout, "  {}", format!("+{}", action.points).with(Color::Yellow))?;
                }
            }
            _ => {}
        }
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
