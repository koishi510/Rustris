use crossterm::style::{Color, Stylize};
use crossterm::{cursor, execute};
use std::io::{self, Write};

use crate::game::Game;
use crate::net::BoardSnapshot;
use crate::game::piece::*;

use super::{color_for, draw_board_cell, draw_full_board_overlay, draw_piece_preview, draw_right_panel, draw_title, draw_title_padded, left_panel_pad, menu_item, BoardRenderState, LEFT_W};

pub fn draw_versus(
    stdout: &mut io::Stdout,
    game: &Game,
    opponent: &Option<BoardSnapshot>,
    pending_garbage: u32,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;

    let state = BoardRenderState::from_game(game);

    let garbage_bar_height = (pending_garbage as usize).min(BOARD_HEIGHT);
    let bar_start_row = BOARD_HEIGHT - garbage_bar_height;

    const VERSUS_TITLE_PAD: usize = 15;
    draw_title_padded(stdout, VERSUS_TITLE_PAD)?;

    write!(stdout, "{:LEFT_W$}╔", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╦═╦")?;
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
                    draw_piece_preview(stdout, game.next_queue[slot], in_slot as i32)?;
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
            draw_board_cell(stdout, &game.board, row, col, &state)?;
        }

        if row >= bar_start_row && garbage_bar_height > 0 {
            write!(stdout, "║{}║", "█".with(Color::Red))?;
        } else {
            write!(stdout, "║ ║")?;
        }

        if let Some(snap) = opponent {
            for col in 0..BOARD_WIDTH {
                let idx = row * BOARD_WIDTH + col;
                let cell = snap.board.get(idx).copied().unwrap_or(0);

                let is_current = snap
                    .current_cells
                    .iter()
                    .any(|&(r, c)| r as usize == row && c as usize == col);

                if is_current {
                    let opp_color = if snap.current_kind < 7 {
                        PIECE_COLORS[snap.current_kind]
                    } else {
                        Color::White
                    };
                    write!(stdout, "{}", "██".with(opp_color))?;
                } else if cell == EMPTY {
                    write!(stdout, "  ")?;
                } else {
                    write!(stdout, "{}", "██".with(color_for(cell)))?;
                }
            }
        } else {
            for _ in 0..BOARD_WIDTH {
                write!(stdout, "  ")?;
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
    write!(stdout, "╩═╩")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╝\x1b[K\r\n")?;

    write!(stdout, "\x1b[J")?;
    stdout.flush()?;
    Ok(())
}

pub fn draw_versus_lobby(
    stdout: &mut io::Stdout,
    is_host: bool,
    status_lines: &[&str],
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;
    let title = if is_host { "HOST" } else { "CLIENT" };

    let mut content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", title, width = inner_w)),
        None,
    ];

    for line in status_lines {
        let truncated = &line[..line.len().min(inner_w)];
        content.push(Some(format!("{:^width$}", truncated, width = inner_w)));
    }

    content.push(None);
    content.push(Some(format!(
        "{:^width$}",
        "ESC to cancel",
        width = inner_w
    )));
    content.push(None);

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_versus_countdown(stdout: &mut io::Stdout, count: u8) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;
    let text = if count == 0 {
        "...".to_string()
    } else {
        count.to_string()
    };

    let content: Vec<Option<String>> = vec![
        None,
        None,
        None,
        Some(format!(
            "{}",
            format!("{:^width$}", text, width = inner_w)
                .as_str()
                .with(Color::Yellow)
        )),
        None,
        None,
    ];

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_versus_result(
    stdout: &mut io::Stdout,
    won: bool,
    selected: usize,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let title = if won { "YOU WIN!" } else { "YOU LOSE" };
    let title_color = if won { Color::Yellow } else { Color::Red };

    let content: Vec<Option<String>> = vec![
        None,
        Some(format!(
            "{}",
            format!("{:^width$}", title, width = inner_w)
                .as_str()
                .with(title_color)
        )),
        None,
        Some(menu_item("Rematch", selected == 0, inner_w)),
        Some(menu_item("Menu", selected == 1, inner_w)),
        Some(menu_item("Quit", selected == 2, inner_w)),
        None,
    ];

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_versus_waiting_rematch(stdout: &mut io::Stdout) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let content: Vec<Option<String>> = vec![
        None,
        None,
        Some(format!("{:^width$}", "Waiting...", width = inner_w)),
        None,
        Some(format!(
            "{:^width$}",
            "ESC to cancel",
            width = inner_w
        )),
        None,
    ];

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_versus_forfeit(
    stdout: &mut io::Stdout,
    selected: usize,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", "FORFEIT?", width = inner_w)),
        None,
        Some(menu_item("Continue", selected == 0, inner_w)),
        Some(menu_item("Forfeit", selected == 1, inner_w)),
        None,
    ];

    draw_full_board_overlay(stdout, &content)
}
