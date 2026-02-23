use crossterm::{cursor, execute, style::{Color, Stylize}};
use std::io::{self, Write};
use std::time::Duration;

use crate::game::Game;
use crate::piece::*;

const LEFT_W: usize = 12;
const TITLE_HEIGHT: u16 = 7;

fn color_for(id: u8) -> Color {
    PIECE_COLORS[(id - 1) as usize]
}

fn draw_piece_preview(
    stdout: &mut io::Stdout,
    kind: usize,
    preview_row: i32,
) -> io::Result<()> {
    let blocks = &PIECE_STATES[kind][0];
    let color = PIECE_COLORS[kind];
    let min_row = blocks.iter().map(|b| b[0]).min().unwrap();
    write!(stdout, "  ")?;
    for preview_col in 0..4i32 {
        let mut found = false;
        for b in blocks {
            if b[0] - min_row == preview_row && b[1] + 1 == preview_col {
                write!(stdout, "{}", "██".with(color))?;
                found = true;
                break;
            }
        }
        if !found {
            write!(stdout, "  ")?;
        }
    }
    Ok(())
}

fn draw_title(stdout: &mut io::Stdout) -> io::Result<()> {
    const LETTERS: [(Color, [&str; 6]); 6] = [
        // T
        (Color::Red, [
            "  ██████╗",
            "  ╚═██╔═╝",
            "    ██║  ",
            "    ██║  ",
            "    ██║  ",
            "    ╚═╝  ",
        ]),
        // E
        (Color::DarkYellow, [
            "██████╗",
            "██╔═══╝",
            "█████╗ ",
            "██╔══╝ ",
            "██████╗",
            "╚═════╝",
        ]),
        // T
        (Color::Yellow, [
            "██████╗",
            "╚═██╔═╝",
            "  ██║  ",
            "  ██║  ",
            "  ██║  ",
            "  ╚═╝  ",
        ]),
        // R
        (Color::Green, [
            "█████╗ ",
            "██╔═██╗",
            "█████╔╝",
            "██╔═██╗",
            "██║ ██║",
            "╚═╝ ╚═╝",
        ]),
        // I
        (Color::Blue, [
            "██╗",
            "██║",
            "██║",
            "██║",
            "██║",
            "╚═╝",
        ]),
        // S
        (Color::Magenta, [
            " █████╗",
            "██╔═══╝",
            "╚████╗ ",
            " ╚══██╗",
            "█████╔╝",
            "╚════╝ ",
        ]),
    ];

    for row in 0..6 {
        write!(stdout, "  ")?;
        for (color, letter) in &LETTERS {
            write!(stdout, "{}", letter[row].with(*color))?;
        }
        write!(stdout, "\x1b[K\r\n")?;
    }
    write!(stdout, "\r\n")?;
    Ok(())
}

fn left_panel_pad(stdout: &mut io::Stdout, visual_len: usize) -> io::Result<()> {
    if visual_len < LEFT_W {
        write!(stdout, "{:width$}", "", width = LEFT_W - visual_len)?;
    }
    Ok(())
}

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
    if animating {
        ghost_cells = [(-1, -1); 4];
        current_cells = [(-1, -1); 4];
        current_color = Color::White;
    } else {
        let ghost_row = game.ghost_row();
        ghost_cells = {
            let mut g = game.current.clone();
            g.row = ghost_row;
            g.cells()
        };
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
            0 => {
                write!(stdout, "{:<LEFT_W$}", "  NEXT:")?;
            }
            2..=19 => {
                let offset = row - 2;
                let slot = offset / 3;
                let in_slot = offset % 3;
                if slot < NEXT_COUNT && in_slot < 2 {
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

        // Board
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

        // Right panel
        write!(stdout, "║")?;
        match row {
            0 => {
                if game.hold_used {
                    write!(stdout, "  {}", "HOLD:".with(Color::DarkGrey))?;
                } else {
                    write!(stdout, "  HOLD:")?;
                }
            }
            2 | 3 => {
                let pr = (row - 2) as i32;
                if let Some(kind) = game.hold {
                    draw_piece_preview(stdout, kind, pr)?;
                }
            }
            5 => write!(stdout, "  SCORE: {}", game.score)?,
            6 => write!(stdout, "  LINES: {}", game.lines)?,
            7 => write!(stdout, "  LEVEL: {}", game.level)?,
            9 => {
                if show_action {
                    let action = game.last_action.as_ref().unwrap();
                    write!(stdout, "  {}", action.label.as_str().with(Color::Yellow))?;
                }
            }
            10 => {
                if show_action {
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

pub fn draw_empty_board(stdout: &mut io::Stdout) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    write!(stdout, "{:LEFT_W$}╔", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╗\x1b[K\r\n")?;

    for _ in 0..BOARD_HEIGHT {
        write!(stdout, "{:LEFT_W$}║", "")?;
        for _ in 0..BOARD_WIDTH {
            write!(stdout, "  ")?;
        }
        write!(stdout, "║\x1b[K\r\n")?;
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

pub fn draw_game_over(stdout: &mut io::Stdout, game: &Game) -> io::Result<()> {
    let x = LEFT_W as u16;
    let y = TITLE_HEIGHT + BOARD_HEIGHT as u16 / 2 - 5;

    let inner_w = BOARD_WIDTH * 2;
    let border = "═".repeat(inner_w);
    let border_line = format!("╠{}╣", border);
    let empty_line = format!("║{:^width$}║", "", width = inner_w);
    let title_line = format!("║{:^width$}║", "GAME  OVER", width = inner_w);
    let score_line = format!("║{:^width$}║", format!("SCORE: {}", game.score), width = inner_w);
    let lines_line = format!("║{:^width$}║", format!("LINES: {}", game.lines), width = inner_w);
    let level_line = format!("║{:^width$}║", format!("LEVEL: {}", game.level), width = inner_w);
    let hint_line = format!("║{:^width$}║", "[R] Retry [Q] Quit", width = inner_w);

    let rows: Vec<String> = vec![
        border_line.clone(),
        empty_line.clone(),
        title_line,
        empty_line.clone(),
        score_line,
        lines_line,
        level_line,
        empty_line.clone(),
        hint_line,
        empty_line.clone(),
        border_line,
    ];

    for (i, row) in rows.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(x, y + i as u16))?;
        write!(stdout, "{}", row.as_str().with(Color::White))?;
    }

    stdout.flush()?;
    Ok(())
}

pub fn draw_pause(stdout: &mut io::Stdout, bgm_on: bool, sfx_on: bool) -> io::Result<()> {
    let x = LEFT_W as u16;
    let y = TITLE_HEIGHT + BOARD_HEIGHT as u16 / 2 - 4;

    let inner_w = BOARD_WIDTH * 2;
    let border = "═".repeat(inner_w);
    let border_line = format!("╠{}╣", border);
    let empty_line = format!("║{:^width$}║", "", width = inner_w);
    let title_line = format!("║{:^width$}║", "PAUSED", width = inner_w);
    let hint_line = format!("║{:^width$}║", "[Esc] Resume", width = inner_w);
    let quit_line = format!("║{:^width$}║", "[Q] Quit", width = inner_w);
    let bgm_status = if bgm_on { "ON" } else { "OFF" };
    let sfx_status = if sfx_on { "ON" } else { "OFF" };
    let bgm_line = format!(
        "║{:^width$}║",
        format!("[M] BGM: {}", bgm_status),
        width = inner_w
    );
    let sfx_line = format!(
        "║{:^width$}║",
        format!("[N] SFX: {}", sfx_status),
        width = inner_w
    );

    let rows: Vec<String> = vec![
        border_line.clone(),
        empty_line.clone(),
        title_line,
        empty_line.clone(),
        hint_line,
        quit_line,
        bgm_line,
        sfx_line,
        empty_line.clone(),
        border_line,
    ];

    for (i, row) in rows.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(x, y + i as u16))?;
        write!(stdout, "{}", row.as_str().with(Color::White))?;
    }

    stdout.flush()?;
    Ok(())
}

pub fn draw_level_select(
    stdout: &mut io::Stdout,
    level: u32,
    bgm_on: bool,
    sfx_on: bool,
) -> io::Result<()> {
    let x = LEFT_W as u16;
    let y = TITLE_HEIGHT + BOARD_HEIGHT as u16 / 2 - 6;

    let inner_w = BOARD_WIDTH * 2;
    let border = "═".repeat(inner_w);
    let border_line = format!("╠{}╣", border);
    let empty_line = format!("║{:^width$}║", "", width = inner_w);
    let title_line = format!("║{:^width$}║", "SELECT LEVEL", width = inner_w);
    let level_line = format!("║{:^width$}║", format!("< {} >", level), width = inner_w);
    let hint1_line = format!("║{:^width$}║", "↑/↓ to change", width = inner_w);
    let hint2_line = format!("║{:^width$}║", "[Enter] Start", width = inner_w);
    let hint3_line = format!("║{:^width$}║", "[Q] Quit", width = inner_w);
    let bgm_status = if bgm_on { "ON" } else { "OFF" };
    let sfx_status = if sfx_on { "ON" } else { "OFF" };
    let bgm_line = format!(
        "║{:^width$}║",
        format!("[M] BGM: {}", bgm_status),
        width = inner_w
    );
    let sfx_line = format!(
        "║{:^width$}║",
        format!("[N] SFX: {}", sfx_status),
        width = inner_w
    );

    let rows: Vec<String> = vec![
        border_line.clone(),
        empty_line.clone(),
        title_line,
        empty_line.clone(),
        level_line,
        empty_line.clone(),
        hint1_line,
        hint2_line,
        hint3_line,
        bgm_line,
        sfx_line,
        empty_line.clone(),
        border_line,
    ];

    for (i, row) in rows.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(x, y + i as u16))?;
        write!(stdout, "{}", row.as_str().with(Color::White))?;
    }

    stdout.flush()?;
    Ok(())
}
