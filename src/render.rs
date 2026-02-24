use crossterm::{cursor, execute, style::{Color, Stylize}};
use std::io::{self, Write};
use std::time::Duration;

use crate::game::{Game, GameMode};
use crate::piece::*;
use crate::settings::Settings;

const LEFT_W: usize = 12;

fn centered_line(text: &str, selected: bool, inner_w: usize, dim: bool) -> String {
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

fn menu_item(label: &str, selected: bool, inner_w: usize) -> String {
    centered_line(label, selected, inner_w, false)
}

fn settings_value(label: &str, value: &str, selected: bool, inner_w: usize) -> String {
    let formatted = format!("{:>5}: < {:^4} >", label, value);
    centered_line(&formatted, selected, inner_w, false)
}

fn settings_toggle(label: &str, on: bool, selected: bool, inner_w: usize) -> String {
    let state = if on { "ON" } else { "OFF" };
    let formatted = format!("{:>5}: {:<8}", label, state);
    centered_line(&formatted, selected, inner_w, false)
}

fn settings_value_dim(label: &str, value: &str, inner_w: usize) -> String {
    let formatted = format!("{:>5}: {:<8}", label, value);
    centered_line(&formatted, false, inner_w, true)
}

fn settings_toggle_dim(label: &str, on: bool, inner_w: usize) -> String {
    let state = if on { "ON" } else { "OFF" };
    let formatted = format!("{:>5}: {:<8}", label, state);
    centered_line(&formatted, false, inner_w, true)
}

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
            0 => {
                write!(stdout, "{:<LEFT_W$}", "  NEXT:")?;
            }
            2..=19 => {
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
                GameMode::Marathon => write!(stdout, "  SCORE: {}", game.score)?,
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
                GameMode::Marathon => match game.marathon_goal {
                    Some(g) => write!(stdout, "  LINES: {} / {}", game.lines, g)?,
                    None => write!(stdout, "  LINES: {}", game.lines)?,
                },
                GameMode::Sprint => write!(stdout, "  LINES: {} / {}", game.lines, game.sprint_goal)?,
                GameMode::Ultra => write!(stdout, "  SCORE: {}", game.score)?,
            },
            7 => match game.mode {
                GameMode::Marathon | GameMode::Sprint => write!(stdout, "  LEVEL: {}", game.level)?,
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

pub fn draw_game_over(stdout: &mut io::Stdout, game: &Game, selected: usize) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let title = if game.cleared {
        "CLEAR!"
    } else if game.mode == GameMode::Ultra && game.elapsed >= Duration::from_secs(game.ultra_time as u64) {
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
        content.push(Some(format!("{:>9}: {:<9}",
            "TIME", format!("{}:{:02}.{:02}", secs / 60, secs % 60, centis)
        )));
    }
    content.push(Some(format!("{:>9}: {:<9}", "SCORE", game.score)));
    content.push(Some(format!("{:>9}: {:<9}", "LINES", game.lines)));
    content.push(Some(format!("{:>9}: {:<9}", "LEVEL", game.level)));
    content.push(None);
    content.push(Some(menu_item("Retry", selected == 0, inner_w)));
    content.push(Some(menu_item("Menu", selected == 1, inner_w)));
    content.push(Some(menu_item("Quit", selected == 2, inner_w)));
    content.push(None);

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_pause(stdout: &mut io::Stdout, selected: usize) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", "PAUSED", width = inner_w)),
        None,
        Some(menu_item("Resume", selected == 0, inner_w)),
        Some(menu_item("Settings", selected == 1, inner_w)),
        Some(menu_item("Help", selected == 2, inner_w)),
        Some(menu_item("Retry", selected == 3, inner_w)),
        Some(menu_item("Menu", selected == 4, inner_w)),
        Some(menu_item("Quit", selected == 5, inner_w)),
        None,
    ];

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_mode_select(
    stdout: &mut io::Stdout,
    mode: GameMode,
    selected: usize,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let mode_name = match mode {
        GameMode::Marathon => "Marathon",
        GameMode::Sprint => "Sprint",
        GameMode::Ultra => "Ultra",
    };

    let mode_label = format!("< {:^8} >", mode_name);

    let content: Vec<Option<String>> = vec![
        None,
        Some(menu_item(&mode_label, selected == 0, inner_w)),
        None,
        Some(menu_item("Start", selected == 1, inner_w)),
        Some(menu_item("Settings", selected == 2, inner_w)),
        Some(menu_item("Help", selected == 3, inner_w)),
        Some(menu_item("Quit", selected == 4, inner_w)),
        None,
    ];

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_help(stdout: &mut io::Stdout, selected: usize) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", "CONTROLS", width = inner_w)),
        None,
        Some(format!("{:^width$}", format!("{:>6}  {:<9}", "←/→", "Move"), width = inner_w)),
        Some(format!("{:^width$}", format!("{:>6}  {:<9}", "↓", "Soft drop"), width = inner_w)),
        Some(format!("{:^width$}", format!("{:>6}  {:<9}", "Space", "Hard drop"), width = inner_w)),
        Some(format!("{:^width$}", format!("{:>6}  {:<9}", "↑/X", "Rotate CW"), width = inner_w)),
        Some(format!("{:^width$}", format!("{:>6}  {:<9}", "Z", "Rotate CCW"), width = inner_w)),
        Some(format!("{:^width$}", format!("{:>6}  {:<9}", "C", "Hold"), width = inner_w)),
        Some(format!("{:^width$}", format!("{:>6}  {:<9}", "Esc/P", "Pause"), width = inner_w)),
        None,
        Some(menu_item("Back", selected == 0, inner_w)),
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

pub fn draw_settings(
    stdout: &mut io::Stdout,
    settings: &Settings,
    mode: GameMode,
    bgm_on: bool,
    sfx_on: bool,
    selected: usize,
    in_game: bool,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;
    let mc: usize = match mode {
        GameMode::Marathon => 3,
        GameMode::Sprint | GameMode::Ultra => 1,
    };

    let mut content: Vec<Option<String>> = vec![
        Some(format!("{:^width$}", "SETTINGS", width = inner_w)),
        None,
    ];

    if in_game {
        match mode {
            GameMode::Marathon => {
                content.push(Some(settings_value_dim("Level", &settings.level.to_string(), inner_w)));
                let goal_str = match settings.marathon_goal {
                    Some(g) => g.to_string(),
                    None => "None".to_string(),
                };
                content.push(Some(settings_value_dim("Goal", &goal_str, inner_w)));
                let cap_str = match settings.level_cap {
                    Some(c) => c.to_string(),
                    None => "None".to_string(),
                };
                content.push(Some(settings_value_dim("Cap", &cap_str, inner_w)));
            }
            GameMode::Sprint => {
                content.push(Some(settings_value_dim("Goal", &settings.sprint_goal.to_string(), inner_w)));
            }
            GameMode::Ultra => {
                let time_str = format!("{}s", settings.ultra_time);
                content.push(Some(settings_value_dim("Time", &time_str, inner_w)));
            }
        }
        content.push(Some(settings_value_dim("Next", &settings.next_count.to_string(), inner_w)));
        content.push(Some(settings_toggle_dim("Ghost", settings.ghost, inner_w)));
        content.push(Some(settings_toggle_dim("Anim", settings.line_clear_anim, inner_w)));
        content.push(Some(settings_toggle_dim("Bag", settings.bag_randomizer, inner_w)));
        content.push(None);
        content.push(Some(settings_toggle("BGM", bgm_on, selected == 0, inner_w)));
        content.push(Some(settings_toggle("SFX", sfx_on, selected == 1, inner_w)));
        content.push(None);
        content.push(Some(menu_item("Back", selected == 2, inner_w)));
    } else {
        match mode {
            GameMode::Marathon => {
                content.push(Some(settings_value("Level", &settings.level.to_string(), selected == 0, inner_w)));
                let goal_str = match settings.marathon_goal {
                    Some(g) => g.to_string(),
                    None => "None".to_string(),
                };
                content.push(Some(settings_value("Goal", &goal_str, selected == 1, inner_w)));
                let cap_str = match settings.level_cap {
                    Some(c) => c.to_string(),
                    None => "None".to_string(),
                };
                content.push(Some(settings_value("Cap", &cap_str, selected == 2, inner_w)));
            }
            GameMode::Sprint => {
                content.push(Some(settings_value("Goal", &settings.sprint_goal.to_string(), selected == 0, inner_w)));
            }
            GameMode::Ultra => {
                let time_str = format!("{}s", settings.ultra_time);
                content.push(Some(settings_value("Time", &time_str, selected == 0, inner_w)));
            }
        }

        content.push(Some(settings_value("Next", &settings.next_count.to_string(), selected == mc, inner_w)));
        content.push(Some(settings_toggle("Ghost", settings.ghost, selected == mc + 1, inner_w)));
        content.push(Some(settings_toggle("Anim", settings.line_clear_anim, selected == mc + 2, inner_w)));
        content.push(Some(settings_toggle("Bag", settings.bag_randomizer, selected == mc + 3, inner_w)));
        content.push(None);
        content.push(Some(settings_toggle("BGM", bgm_on, selected == mc + 4, inner_w)));
        content.push(Some(settings_toggle("SFX", sfx_on, selected == mc + 5, inner_w)));
        content.push(None);
        content.push(Some(menu_item("Back", selected == mc + 6, inner_w)));
    }

    draw_full_board_overlay(stdout, &content)
}
