use crossterm::{cursor, execute, style::{Color, Stylize}};
use std::io::{self, Write};
use std::time::Duration;

use crate::game::{Game, GameMode};
use crate::piece::*;
use crate::records::Records;
use crate::settings::Settings;

use super::{
    draw_full_board_overlay, draw_title, menu_item, settings_toggle, settings_toggle_dim,
    settings_value, settings_value_dim, LEFT_W,
};

pub fn draw_game_over(
    stdout: &mut io::Stdout,
    game: &Game,
    selected: usize,
    rank: Option<usize>,
) -> io::Result<()> {
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
    if let Some(r) = rank {
        let record_text = format!("NEW RECORD! #{}", r + 1);
        let padded = format!("{:^width$}", record_text, width = inner_w);
        content.push(Some(format!("{}", padded.as_str().with(Color::Yellow))));
    }
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
        GameMode::Endless => "Endless",
    };

    let mode_label = format!("< {:^8} >", mode_name);

    let content: Vec<Option<String>> = vec![
        None,
        Some(menu_item(&mode_label, selected == 0, inner_w)),
        None,
        Some(menu_item("Start", selected == 1, inner_w)),
        Some(menu_item("Settings", selected == 2, inner_w)),
        Some(menu_item("Records", selected == 3, inner_w)),
        Some(menu_item("Help", selected == 4, inner_w)),
        Some(menu_item("Quit", selected == 5, inner_w)),
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
        GameMode::Endless => 2,
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
                content.push(Some(settings_value_dim("Goal", &settings.marathon_goal.to_string(), inner_w)));
                let cap_str = match settings.level_cap {
                    Some(c) => c.to_string(),
                    None => "INF".to_string(),
                };
                content.push(Some(settings_value_dim("Cap", &cap_str, inner_w)));
            }
            GameMode::Endless => {
                content.push(Some(settings_value_dim("Level", &settings.level.to_string(), inner_w)));
                let cap_str = match settings.level_cap {
                    Some(c) => c.to_string(),
                    None => "INF".to_string(),
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
        let lock_str = format!("{:.1}s", settings.lock_delay_ms as f32 / 1000.0);
        content.push(Some(settings_value_dim("Lock", &lock_str, inner_w)));
        let reset_str = match settings.move_reset {
            Some(n) => n.to_string(),
            None => "INF".to_string(),
        };
        content.push(Some(settings_value_dim("Reset", &reset_str, inner_w)));
        content.push(Some(settings_toggle_dim("Ghost", settings.ghost, inner_w)));
        content.push(Some(settings_toggle_dim("Anim", settings.line_clear_anim, inner_w)));
        content.push(Some(settings_toggle_dim("Bag", settings.bag_randomizer, inner_w)));
        content.push(Some(settings_toggle_dim("SRS", settings.srs, inner_w)));
        content.push(Some(settings_toggle_dim("Hold", settings.hold_enabled, inner_w)));
        content.push(None);
        content.push(Some(settings_toggle("BGM", bgm_on, selected == 0, inner_w)));
        content.push(Some(settings_toggle("SFX", sfx_on, selected == 1, inner_w)));
        content.push(None);
        content.push(Some(menu_item("Back", selected == 2, inner_w)));
    } else {
        match mode {
            GameMode::Marathon => {
                content.push(Some(settings_value("Level", &settings.level.to_string(), selected == 0, inner_w)));
                content.push(Some(settings_value("Goal", &settings.marathon_goal.to_string(), selected == 1, inner_w)));
                let cap_str = match settings.level_cap {
                    Some(c) => c.to_string(),
                    None => "INF".to_string(),
                };
                content.push(Some(settings_value("Cap", &cap_str, selected == 2, inner_w)));
            }
            GameMode::Endless => {
                content.push(Some(settings_value("Level", &settings.level.to_string(), selected == 0, inner_w)));
                let cap_str = match settings.level_cap {
                    Some(c) => c.to_string(),
                    None => "INF".to_string(),
                };
                content.push(Some(settings_value("Cap", &cap_str, selected == 1, inner_w)));
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
        let lock_str = format!("{:.1}s", settings.lock_delay_ms as f32 / 1000.0);
        content.push(Some(settings_value("Lock", &lock_str, selected == mc + 1, inner_w)));
        let reset_str = match settings.move_reset {
            Some(n) => n.to_string(),
            None => "INF".to_string(),
        };
        content.push(Some(settings_value("Reset", &reset_str, selected == mc + 2, inner_w)));
        content.push(Some(settings_toggle("Ghost", settings.ghost, selected == mc + 3, inner_w)));
        content.push(Some(settings_toggle("Anim", settings.line_clear_anim, selected == mc + 4, inner_w)));
        content.push(Some(settings_toggle("Bag", settings.bag_randomizer, selected == mc + 5, inner_w)));
        content.push(Some(settings_toggle("SRS", settings.srs, selected == mc + 6, inner_w)));
        content.push(Some(settings_toggle("Hold", settings.hold_enabled, selected == mc + 7, inner_w)));
        content.push(None);
        content.push(Some(settings_toggle("BGM", bgm_on, selected == mc + 8, inner_w)));
        content.push(Some(settings_toggle("SFX", sfx_on, selected == mc + 9, inner_w)));
        content.push(None);
        content.push(Some(menu_item("Back", selected == mc + 10, inner_w)));
    }

    draw_full_board_overlay(stdout, &content)
}

pub fn draw_records(
    stdout: &mut io::Stdout,
    records: &Records,
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
        GameMode::Endless => "Endless",
    };
    let mode_label = format!("< {:^8} >", mode_name);

    let list = match mode {
        GameMode::Marathon => &records.marathon,
        GameMode::Sprint => &records.sprint,
        GameMode::Ultra => &records.ultra,
        GameMode::Endless => &records.endless,
    };

    let separator = "─".repeat(inner_w);

    let mut content: Vec<Option<String>> = Vec::new();
    content.push(Some(menu_item(&mode_label, selected == 0, inner_w)));
    content.push(Some(separator.clone()));

    for i in 0..10 {
        if i < list.len() {
            let r = &list[i];
            let line = match mode {
                GameMode::Sprint => {
                    let t = r.time.unwrap_or(0);
                    let secs = t / 1000;
                    let centis = (t % 1000) / 10;
                    format!(
                        "#{:<2} {}:{:02}.{:02} L{:<2} {:>3}L",
                        i + 1,
                        secs / 60,
                        secs % 60,
                        centis,
                        r.level,
                        r.lines,
                    )
                }
                _ => {
                    format!(
                        "#{:<2} {:>7} L{:<2} {:>3}L",
                        i + 1,
                        r.score,
                        r.level,
                        r.lines,
                    )
                }
            };
            content.push(Some(format!("{:^width$}", line, width = inner_w)));
        } else {
            let line = match mode {
                GameMode::Sprint => {
                    format!("#{:<2} {:>7} L{:<2} {:>3}L", i + 1, "-:--.--", "-", "-")
                }
                _ => {
                    format!("#{:<2} {:>7} L{:<2} {:>3}L", i + 1, "-", "-", "-")
                }
            };
            let padded = format!("{:^width$}", line, width = inner_w);
            content.push(Some(format!("{}", padded.as_str().with(Color::DarkGrey))));
        }
    }

    content.push(Some(separator));
    content.push(Some(menu_item("Back", selected == 1, inner_w)));

    draw_full_board_overlay(stdout, &content)
}
