use crossterm::{cursor, execute, style::{Color, Stylize}};
use std::io::{self, Write};
use std::time::Duration;

use crate::game::{Game, GameMode};
use crate::piece::*;

const LEFT_W: usize = 12;

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
    if animating || game.in_are() {
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
                        write!(stdout, "  TIME: {}:{:02}", secs / 60, secs % 60)?;
                    }
                }
            },
            6 => match game.mode {
                GameMode::Marathon | GameMode::Endless => write!(stdout, "  LINES: {}", game.lines)?,
                GameMode::Sprint => {
                    write!(stdout, "  LINES: {} / 40", game.lines)?;
                }
                GameMode::Ultra => write!(stdout, "  SCORE: {}", game.score)?,
            },
            7 => match game.mode {
                GameMode::Marathon | GameMode::Endless => write!(stdout, "  LEVEL: {}", game.level)?,
                GameMode::Sprint => write!(stdout, "  LEVEL: {}", game.level)?,
                GameMode::Ultra => write!(stdout, "  LINES: {}", game.lines)?,
            },
            8 => match game.mode {
                GameMode::Marathon => {
                    let goal = 150u32.saturating_sub(game.lines);
                    write!(stdout, "  GOAL: {}", goal)?;
                }
                _ => {}
            },
            9 => match game.mode {
                GameMode::Ultra => write!(stdout, "  LEVEL: {}", game.level)?,
                _ => {
                    if show_action {
                        let action = game.last_action.as_ref().unwrap();
                        write!(stdout, "  {}", action.label.as_str().with(Color::Yellow))?;
                    }
                }
            },
            10 => match game.mode {
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
            11 => {
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


fn draw_full_board_overlay(
    stdout: &mut io::Stdout,
    content: &[Option<String>],
) -> io::Result<()> {
    let inner_w = BOARD_WIDTH * 2;
    let start_row = (BOARD_HEIGHT - content.len()) / 2;

    write!(stdout, "{:LEFT_W$}╔", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╗\x1b[K\r\n")?;

    for row in 0..BOARD_HEIGHT {
        write!(stdout, "{:LEFT_W$}║", "")?;
        if row >= start_row && row - start_row < content.len() {
            match &content[row - start_row] {
                Some(text) => write!(stdout, "{}", text)?,
                None => write!(stdout, "{:width$}", "", width = inner_w)?,
            }
        } else {
            write!(stdout, "{:width$}", "", width = inner_w)?;
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
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let title = if game.cleared {
        "CLEAR!"
    } else if game.mode == GameMode::Ultra && game.elapsed >= std::time::Duration::from_secs(120) {
        "TIME'S UP!"
    } else {
        "GAME  OVER"
    };

    let mut content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", title, width = inner_w)),
        None,
    ];

    if game.mode == GameMode::Sprint && game.cleared {
        let secs = game.elapsed.as_secs();
        let centis = game.elapsed.subsec_millis() / 10;
        content.push(Some(format!(
            "{:^width$}",
            format!("TIME: {}:{:02}.{:02}", secs / 60, secs % 60, centis),
            width = inner_w
        )));
    }
    content.push(Some(format!("{:^width$}", format!("SCORE: {}", game.score), width = inner_w)));
    content.push(Some(format!("{:^width$}", format!("LINES: {}", game.lines), width = inner_w)));
    content.push(Some(format!("{:^width$}", format!("LEVEL: {}", game.level), width = inner_w)));
    content.push(None);
    content.push(Some(format!("{:^width$}", "[R] Retry  [Q] Quit", width = inner_w)));
    content.push(None);

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_pause(stdout: &mut io::Stdout, bgm_on: bool, sfx_on: bool) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;
    let bgm_status = if bgm_on { "ON" } else { "OFF" };
    let sfx_status = if sfx_on { "ON" } else { "OFF" };

    let content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", "PAUSED", width = inner_w)),
        None,
        Some(format!("{:^width$}", "[Esc] Resume", width = inner_w)),
        Some(format!("{:^width$}", "[R] Retry", width = inner_w)),
        Some(format!("{:^width$}", "[Q] Quit", width = inner_w)),
        Some(format!("{:^width$}", "[H] Help", width = inner_w)),
        Some(format!("{:^width$}", format!("[M] BGM: {}", bgm_status), width = inner_w)),
        Some(format!("{:^width$}", format!("[N] SFX: {}", sfx_status), width = inner_w)),
        None,
    ];

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_mode_select(
    stdout: &mut io::Stdout,
    mode: GameMode,
    level: u32,
    bgm_on: bool,
    sfx_on: bool,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;
    let bgm_status = if bgm_on { "ON" } else { "OFF" };
    let sfx_status = if sfx_on { "ON" } else { "OFF" };

    let mode_name = match mode {
        GameMode::Marathon => "Marathon",
        GameMode::Sprint => "Sprint",
        GameMode::Ultra => "Ultra",
        GameMode::Endless => "Endless",
    };

    let mut content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", format!("< {} >", mode_name), width = inner_w)),
        None,
    ];

    if mode == GameMode::Marathon || mode == GameMode::Endless {
        content.push(Some(format!("{:^width$}", format!("Level: < {} >", level), width = inner_w)));
    }

    content.push(None);
    content.push(Some(format!("{:^width$}", "←/→ change mode", width = inner_w)));
    if mode == GameMode::Marathon || mode == GameMode::Endless {
        content.push(Some(format!("{:^width$}", "↑/↓ change level", width = inner_w)));
    }
    content.push(Some(format!("{:^width$}", "[Enter] Start", width = inner_w)));
    content.push(Some(format!("{:^width$}", "[H] Help", width = inner_w)));
    content.push(Some(format!("{:^width$}", "[Q] Quit", width = inner_w)));
    content.push(Some(format!("{:^width$}", format!("[M] BGM: {}", bgm_status), width = inner_w)));
    content.push(Some(format!("{:^width$}", format!("[N] SFX: {}", sfx_status), width = inner_w)));
    content.push(None);

    let start_row = (BOARD_HEIGHT - content.len()) / 2;

    write!(stdout, "{:LEFT_W$}╔", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╗\x1b[K\r\n")?;

    for row in 0..BOARD_HEIGHT {
        write!(stdout, "{:LEFT_W$}║", "")?;
        if row >= start_row && row - start_row < content.len() {
            match &content[row - start_row] {
                Some(text) => write!(stdout, "{}", text)?,
                None => write!(stdout, "{:width$}", "", width = inner_w)?,
            }
        } else {
            write!(stdout, "{:width$}", "", width = inner_w)?;
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

pub fn draw_help(stdout: &mut io::Stdout) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", "CONTROLS", width = inner_w)),
        None,
        Some(format!("{:^width$}", "←/→     Move piece", width = inner_w)),
        Some(format!("{:^width$}", "↓       Soft drop ", width = inner_w)),
        Some(format!("{:^width$}", "Space   Hard drop ", width = inner_w)),
        Some(format!("{:^width$}", "↑/X     Rotate CW ", width = inner_w)),
        Some(format!("{:^width$}", "Z       Rotate CCW", width = inner_w)),
        Some(format!("{:^width$}", "C       Hold piece", width = inner_w)),
        Some(format!("{:^width$}", "Esc     Pause     ", width = inner_w)),
        None,
        Some(format!("{:^width$}", "[M] BGM  [N] SFX", width = inner_w)),
        None,
        Some(format!("{:^width$}", "[Esc] Return", width = inner_w)),
        None,
    ];

    let start_row = (BOARD_HEIGHT - content.len()) / 2;

    write!(stdout, "{:LEFT_W$}╔", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╗\x1b[K\r\n")?;

    for row in 0..BOARD_HEIGHT {
        write!(stdout, "{:LEFT_W$}║", "")?;
        if row >= start_row && row - start_row < content.len() {
            match &content[row - start_row] {
                Some(text) => write!(stdout, "{}", text)?,
                None => write!(stdout, "{:width$}", "", width = inner_w)?,
            }
        } else {
            write!(stdout, "{:width$}", "", width = inner_w)?;
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
