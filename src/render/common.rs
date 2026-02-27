use crossterm::style::{Color, Stylize};
use std::io::{self, Write};
use std::time::Duration;

use crate::game::{Game, GameMode};
use crate::game::piece::*;

pub(crate) const LEFT_W: usize = 12;

fn display_width(s: &str) -> usize {
    s.chars().count()
}

pub(crate) fn centered_line(text: &str, selected: bool, inner_w: usize, dim: bool) -> String {
    let prefix = if selected { "> " } else { "  " };
    let prefix_len = prefix.len();
    let text_w = display_width(text);
    let total_pad = inner_w.saturating_sub(text_w);
    let left_pad = total_pad / 2;
    let right_pad = total_pad - left_pad;
    let overflow = prefix_len.saturating_sub(left_pad);
    let line = format!(
        "{:ls$}{}{}{:rs$}",
        "", prefix, text, "",
        ls = left_pad.saturating_sub(prefix_len),
        rs = right_pad.saturating_sub(overflow),
    );
    if dim {
        format!("{}", line.as_str().with(Color::DarkGrey))
    } else if selected {
        format!("{}", line.as_str().with(Color::Yellow))
    } else {
        line
    }
}

pub(crate) fn menu_item(label: &str, selected: bool, inner_w: usize) -> String {
    centered_line(label, selected, inner_w, false)
}

pub(crate) fn input_item(text: &str, selected: bool, indent: usize, inner_w: usize) -> String {
    if selected {
        let prefix = "> ";
        let text_w = display_width(text);
        let right_pad = inner_w.saturating_sub(indent + prefix.len() + text_w);
        let line = format!("{:ind$}{}{}{:rs$}", "", prefix, text, "", ind = indent, rs = right_pad);
        format!("{}", line.as_str().with(Color::Yellow))
    } else {
        format!("{:^width$}", text, width = inner_w)
    }
}

pub(crate) fn settings_value(label: &str, value: &str, selected: bool, inner_w: usize) -> String {
    let formatted = format!("{:>5}:  < {:^4} >", label, value);
    centered_line(&formatted, selected, inner_w, false)
}

pub(crate) fn settings_toggle(label: &str, on: bool, selected: bool, inner_w: usize) -> String {
    let state = if on { "ON" } else { "OFF" };
    let formatted = format!("{:>5}:  < {:^4} >", label, state);
    centered_line(&formatted, selected, inner_w, false)
}

pub(crate) fn settings_value_dim(label: &str, value: &str, inner_w: usize) -> String {
    let formatted = format!("{:>5}:  < {:^4} >", label, value);
    centered_line(&formatted, false, inner_w, true)
}

pub(crate) fn settings_toggle_dim(label: &str, on: bool, inner_w: usize) -> String {
    let state = if on { "ON" } else { "OFF" };
    let formatted = format!("{:>5}:  < {:^4} >", label, state);
    centered_line(&formatted, false, inner_w, true)
}

pub(crate) fn draw_board_top(stdout: &mut io::Stdout) -> io::Result<()> {
    write!(stdout, "{:LEFT_W$}╔", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╗\x1b[K\r\n")
}

pub(crate) fn draw_board_bottom(stdout: &mut io::Stdout) -> io::Result<()> {
    write!(stdout, "{:LEFT_W$}╚", "")?;
    for _ in 0..BOARD_WIDTH {
        write!(stdout, "══")?;
    }
    write!(stdout, "╝\x1b[K\r\n")
}

pub(crate) fn format_time(d: Duration) -> String {
    let secs = d.as_secs();
    let centis = d.subsec_millis() / 10;
    format!("{}:{:02}.{:02}", secs / 60, secs % 60, centis)
}

pub(crate) fn color_for(id: u8) -> Color {
    if id == GARBAGE_CELL {
        Color::DarkGrey
    } else if (1..=7).contains(&id) {
        PIECE_COLORS[(id - 1) as usize]
    } else {
        Color::White
    }
}

pub(crate) struct BoardRenderState {
    pub ghost_cells: [(i32, i32); 4],
    pub current_cells: [(i32, i32); 4],
    pub current_color: Color,
    pub anim_rows: Vec<usize>,
    pub anim_phase: u8,
}

impl BoardRenderState {
    pub fn from_game(game: &Game) -> Self {
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
                    let mut g = game.current;
                    g.row = ghost_row;
                    g.cells()
                };
            } else {
                ghost_cells = [(-1, -1); 4];
            }
            current_cells = game.current.cells();
            current_color = PIECE_COLORS[game.current.kind];
        }

        Self {
            ghost_cells,
            current_cells,
            current_color,
            anim_rows,
            anim_phase,
        }
    }
}

pub(crate) fn draw_board_cell(
    stdout: &mut io::Stdout,
    board: &[[u8; BOARD_WIDTH]; BOARD_HEIGHT],
    board_row: usize,
    col: usize,
    state: &BoardRenderState,
) -> io::Result<()> {
    if state.anim_rows.contains(&board_row) {
        match state.anim_phase {
            0 => write!(stdout, "{}", "██".with(Color::White))?,
            1 => write!(stdout, "{}", "▓▓".with(Color::DarkGrey))?,
            _ => write!(stdout, "  ")?,
        }
    } else if state.current_cells.contains(&(board_row as i32, col as i32)) {
        write!(stdout, "{}", "██".with(state.current_color))?;
    } else if state.ghost_cells.contains(&(board_row as i32, col as i32))
        && board[board_row][col] == EMPTY
    {
        write!(stdout, "{}", "░░".with(state.current_color))?;
    } else {
        let id = board[board_row][col];
        if id == EMPTY {
            write!(stdout, "  ")?;
        } else {
            write!(stdout, "{}", "██".with(color_for(id)))?;
        }
    }
    Ok(())
}

pub(crate) fn draw_piece_preview(
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

pub(crate) fn draw_title(stdout: &mut io::Stdout) -> io::Result<()> {
    draw_title_padded(stdout, 4)
}

pub(crate) fn draw_title_padded(stdout: &mut io::Stdout, pad: usize) -> io::Result<()> {
    const LETTERS: [(Color, [&str; 6]); 6] = [
        // T
        (Color::Red, [
            "██████╗",
            "╚═██╔═╝",
            "  ██║  ",
            "  ██║  ",
            "  ██║  ",
            "  ╚═╝  ",
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
        write!(stdout, "{:pad$}", "")?;
        for (color, letter) in &LETTERS {
            write!(stdout, "{}", letter[row].with(*color))?;
        }
        write!(stdout, "\x1b[K\r\n")?;
    }
    write!(stdout, "\r\n")?;
    Ok(())
}

pub(crate) fn left_panel_pad(stdout: &mut io::Stdout, visual_len: usize) -> io::Result<()> {
    if visual_len < LEFT_W {
        write!(stdout, "{:width$}", "", width = LEFT_W - visual_len)?;
    }
    Ok(())
}

pub(crate) fn draw_full_board_overlay(
    stdout: &mut io::Stdout,
    content: &[Option<String>],
) -> io::Result<()> {
    let inner_w = BOARD_WIDTH * 2;
    let start_row = (VISIBLE_HEIGHT - content.len()) / 2;

    draw_board_top(stdout)?;

    for row in 0..VISIBLE_HEIGHT {
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

    draw_board_bottom(stdout)?;

    write!(stdout, "\x1b[J")?;
    stdout.flush()?;
    Ok(())
}

pub(crate) fn draw_right_panel(stdout: &mut io::Stdout, game: &Game, row: usize) -> io::Result<()> {
    let show_action = game.last_action.is_some()
        && game.last_action_time.elapsed() < Duration::from_secs(3);

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
            GameMode::Marathon | GameMode::Endless | GameMode::Versus => write!(stdout, "  SCORE: {}", game.score)?,
            GameMode::Sprint => {
                write!(stdout, "  TIME: {}", format_time(game.elapsed))?;
            }
            GameMode::Ultra => {
                if let Some(rem) = game.time_remaining() {
                    write!(stdout, "  TIME: {}", format_time(rem))?;
                }
            }
        },
        6 => match game.mode {
            GameMode::Marathon => write!(stdout, "  LINES: {} / {}", game.lines, game.marathon_goal)?,
            GameMode::Endless | GameMode::Versus => write!(stdout, "  LINES: {}", game.lines)?,
            GameMode::Sprint => write!(stdout, "  LINES: {} / {}", game.lines, game.sprint_goal)?,
            GameMode::Ultra => write!(stdout, "  SCORE: {}", game.score)?,
        },
        7 => match game.mode {
            GameMode::Marathon | GameMode::Sprint | GameMode::Endless | GameMode::Versus => write!(stdout, "  LEVEL: {}", game.level)?,
            GameMode::Ultra => write!(stdout, "  LINES: {}", game.lines)?,
        },
        8 => match game.mode {
            GameMode::Ultra => write!(stdout, "  LEVEL: {}", game.level)?,
            _ => {
                if let Some(action) = show_action.then_some(game.last_action.as_ref()).flatten() {
                    write!(stdout, "  {}", action.label.as_str().with(Color::Yellow))?;
                }
            }
        },
        9 => match game.mode {
            GameMode::Ultra => {
                if let Some(action) = show_action.then_some(game.last_action.as_ref()).flatten() {
                    write!(stdout, "  {}", action.label.as_str().with(Color::Yellow))?;
                }
            }
            _ => {
                if let Some(action) = show_action.then_some(game.last_action.as_ref()).flatten() {
                    write!(stdout, "  {}", format!("+{}", action.points).with(Color::Yellow))?;
                }
            }
        },
        10 => {
            if let Some(action) = show_action.then_some(game.last_action.as_ref()).flatten() {
                if game.mode == GameMode::Ultra {
                    write!(stdout, "  {}", format!("+{}", action.points).with(Color::Yellow))?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
