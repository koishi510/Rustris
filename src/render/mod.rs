mod board;
mod menus;

pub use board::draw;
pub use menus::{draw_game_over, draw_help, draw_mode_select, draw_pause, draw_records, draw_settings};

use crossterm::style::{Color, Stylize};
use std::io::{self, Write};

use crate::piece::*;

pub(crate) const LEFT_W: usize = 12;

pub(crate) fn centered_line(text: &str, selected: bool, inner_w: usize, dim: bool) -> String {
    let prefix = if selected { "> " } else { "  " };
    let prefix_len = prefix.len();
    let total_pad = inner_w.saturating_sub(text.len());
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

pub(crate) fn color_for(id: u8) -> Color {
    PIECE_COLORS[(id - 1) as usize]
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
        write!(stdout, "    ")?;
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
