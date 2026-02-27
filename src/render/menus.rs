use crossterm::{cursor, execute, style::{Color, Stylize}};
use std::io::{self, Write};
use std::time::Duration;

use crate::game::{Game, GameMode, format_option_or_inf};
use crate::game::piece::*;
use crate::game::records::Records;
use crate::game::settings::Settings;

use super::{
    draw_board_bottom, draw_board_top, draw_full_board_overlay, draw_title, format_time,
    menu_item, settings_toggle, settings_toggle_dim, settings_value, settings_value_dim, LEFT_W,
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

    let title_color = if game.cleared {
        Color::Yellow
    } else {
        Color::Red
    };

    let mut content: Vec<Option<String>> = vec![
        None,
        Some(format!(
            "{}",
            format!("{:^width$}", title, width = inner_w)
                .as_str()
                .with(title_color)
        )),
        None,
    ];

    if game.mode == GameMode::Sprint && game.cleared {
        content.push(Some(format!("{:>9}: {:<9}",
            "TIME", format_time(game.elapsed)
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
        GameMode::Versus => "Versus",
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
    let mc = mode.setting_count();

    let sv = |label: &str, value: &str, idx: usize| {
        if in_game {
            settings_value_dim(label, value, inner_w)
        } else {
            settings_value(label, value, selected == idx, inner_w)
        }
    };
    let st = |label: &str, on: bool, idx: usize| {
        if in_game {
            settings_toggle_dim(label, on, inner_w)
        } else {
            settings_toggle(label, on, selected == idx, inner_w)
        }
    };

    let mut content: Vec<Option<String>> = vec![
        Some(format!("{:^width$}", "SETTINGS", width = inner_w)),
        None,
    ];

    match mode {
        GameMode::Marathon => {
            content.push(Some(sv("Level", &settings.level.to_string(), 0)));
            content.push(Some(sv("Goal", &settings.marathon_goal.to_string(), 1)));
            content.push(Some(sv("Cap", &format_option_or_inf(settings.level_cap), 2)));
        }
        GameMode::Endless => {
            content.push(Some(sv("Level", &settings.level.to_string(), 0)));
            content.push(Some(sv("Cap", &format_option_or_inf(settings.level_cap), 1)));
        }
        GameMode::Sprint => {
            content.push(Some(sv("Goal", &settings.sprint_goal.to_string(), 0)));
        }
        GameMode::Ultra => {
            let time_str = format!("{}s", settings.ultra_time);
            content.push(Some(sv("Time", &time_str, 0)));
        }
        GameMode::Versus => {
            content.push(Some(sv("Level", &settings.level.to_string(), 0)));
        }
    }

    content.push(Some(sv("Next", &settings.next_count.to_string(), mc)));
    let lock_str = format!("{:.1}s", settings.lock_delay_ms as f32 / 1000.0);
    content.push(Some(sv("Lock", &lock_str, mc + 1)));
    content.push(Some(sv("Reset", &format_option_or_inf(settings.move_reset), mc + 2)));
    content.push(Some(st("Ghost", settings.ghost, mc + 3)));
    content.push(Some(st("Anim", settings.line_clear_anim, mc + 4)));
    content.push(Some(st("Bag", settings.bag_randomizer, mc + 5)));
    content.push(Some(st("SRS", settings.srs_enabled, mc + 6)));
    content.push(Some(st("Hold", settings.hold_enabled, mc + 7)));
    content.push(None);

    if in_game {
        content.push(Some(settings_toggle("BGM", bgm_on, selected == 0, inner_w)));
        content.push(Some(settings_toggle("SFX", sfx_on, selected == 1, inner_w)));
        content.push(None);
        content.push(Some(menu_item("Back", selected == 2, inner_w)));
    } else {
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
        GameMode::Versus => "Versus",
    };
    let mode_label = format!("< {:^8} >", mode_name);

    let list = match mode {
        GameMode::Marathon => &records.marathon,
        GameMode::Sprint => &records.sprint,
        GameMode::Ultra => &records.ultra,
        GameMode::Endless | GameMode::Versus => &records.endless,
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
