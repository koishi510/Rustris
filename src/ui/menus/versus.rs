use crossterm::event::KeyCode;
use crossterm::style::{Color, Stylize};
use crossterm::{cursor, execute};
use std::io;

use crate::audio::{self, Sfx};
use crate::game::piece::BOARD_WIDTH;
use crate::render;
use crate::game::settings::Settings;
use crate::ui::{menu_nav, play_menu_sfx, read_key};

pub enum VersusAction {
    Host(u16),
    Join(String),
    Back,
}

enum InputResult {
    Confirm(String),
    Back,
    Cancel,
}

fn draw_versus_menu(
    stdout: &mut io::Stdout,
    selected: usize,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    render::draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", "VERSUS MODE", width = inner_w)),
        None,
        Some(render::menu_item("Host Game", selected == 0, inner_w)),
        Some(render::menu_item("Join Game", selected == 1, inner_w)),
        None,
        Some(render::menu_item("Back", selected == 2, inner_w)),
        None,
    ];

    render::draw_full_board_overlay(stdout, &content)
}

fn draw_input_screen(
    stdout: &mut io::Stdout,
    title: &str,
    label: &str,
    input: &str,
    error: &str,
    selected: usize,
    indent: usize,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    render::draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let input_text = if selected == 0 {
        format!("{}█", input)
    } else {
        input.to_string()
    };
    let mut content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", title, width = inner_w)),
        None,
        Some(format!("{:^width$}", label, width = inner_w)),
        Some(render::input_item(&input_text, selected == 0, indent, inner_w)),
    ];

    if !error.is_empty() {
        let truncated = &error[..error.len().min(inner_w)];
        content.push(Some(format!(
            "{}",
            format!("{:^width$}", truncated, width = inner_w)
                .as_str()
                .with(Color::Red)
        )));
    }

    content.push(None);
    content.push(Some(render::menu_item("Confirm", selected == 1, inner_w)));
    content.push(Some(render::menu_item("Back", selected == 2, inner_w)));
    content.push(Some(render::menu_item("Menu", selected == 3, inner_w)));
    content.push(None);

    render::draw_full_board_overlay(stdout, &content)
}

pub fn run_versus_menu(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    _settings: &mut Settings,
) -> io::Result<VersusAction> {
    let mut sel: usize = 0;
    let count: usize = 3;

    loop {
        draw_versus_menu(stdout, sel)?;

        if let Some(code) = read_key()? {
            match code {
                KeyCode::Up | KeyCode::Down => {
                    sel = menu_nav(sel, count, code);
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Enter => match sel {
                    0 => {
                        play_menu_sfx(music, Sfx::MenuSelect);
                        match run_port_input(stdout, music)? {
                            InputResult::Confirm(port) => {
                                if let Ok(p) = port.parse::<u16>() {
                                    return Ok(VersusAction::Host(p));
                                }
                            }
                            InputResult::Back => continue,
                            InputResult::Cancel => return Ok(VersusAction::Back),
                        }
                    }
                    1 => {
                        play_menu_sfx(music, Sfx::MenuSelect);
                        match run_addr_input(stdout, music)? {
                            InputResult::Confirm(addr) => return Ok(VersusAction::Join(addr)),
                            InputResult::Back => continue,
                            InputResult::Cancel => return Ok(VersusAction::Back),
                        }
                    }
                    2 => {
                        play_menu_sfx(music, Sfx::MenuBack);
                        return Ok(VersusAction::Back);
                    }
                    _ => {}
                },
                KeyCode::Esc => {
                    play_menu_sfx(music, Sfx::MenuBack);
                    return Ok(VersusAction::Back);
                }
                _ => {}
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_text_input(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    title: &str,
    label: &str,
    default: &str,
    max_len: usize,
    indent: usize,
    char_filter: fn(char) -> bool,
    validate: &dyn Fn(&str) -> Result<(), String>,
) -> io::Result<InputResult> {
    let mut input = default.to_string();
    let mut error = String::new();
    let mut sel: usize = 0;
    let count: usize = 4;

    loop {
        draw_input_screen(stdout, title, label, &input, &error, sel, indent)?;

        if let Some(code) = read_key()? {
            match code {
                KeyCode::Up | KeyCode::Down => {
                    sel = menu_nav(sel, count, code);
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                KeyCode::Char(c) if sel == 0 && char_filter(c) => {
                    if input.len() < max_len {
                        input.push(c);
                        error.clear();
                    }
                }
                KeyCode::Backspace if sel == 0 => {
                    input.pop();
                    error.clear();
                }
                KeyCode::Enter => match sel {
                    0 | 1 => match validate(&input) {
                        Ok(()) => {
                            play_menu_sfx(music, Sfx::MenuSelect);
                            return Ok(InputResult::Confirm(input));
                        }
                        Err(msg) => {
                            error = msg;
                        }
                    },
                    2 => {
                        play_menu_sfx(music, Sfx::MenuBack);
                        return Ok(InputResult::Back);
                    }
                    _ => {
                        play_menu_sfx(music, Sfx::MenuBack);
                        return Ok(InputResult::Cancel);
                    }
                },
                KeyCode::Esc => {
                    play_menu_sfx(music, Sfx::MenuBack);
                    return Ok(InputResult::Back);
                }
                _ => {}
            }
        }
    }
}

fn run_port_input(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<InputResult> {
    run_text_input(
        stdout,
        music,
        "HOST GAME",
        "Port",
        "21711",
        5,
        6,
        |c| c.is_ascii_digit(),
        &|s| match s.parse::<u16>() {
            Ok(port) if port > 0 => Ok(()),
            _ => Err("Invalid port".to_string()),
        },
    )
}

fn run_addr_input(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<InputResult> {
    'ip: loop {
        let ip = match run_text_input(
            stdout,
            music,
            "JOIN GAME",
            "IP Address",
            "127.0.0.1",
            15,
            1,
            |c| c.is_ascii_digit() || c == '.',
            &|s| {
                if s.is_empty() {
                    Err("Enter an address".to_string())
                } else if s.split('.').count() != 4
                    || s.split('.').any(|p| p.is_empty() || p.parse::<u8>().is_err())
                {
                    Err("Invalid IP".to_string())
                } else {
                    Ok(())
                }
            },
        )? {
            InputResult::Confirm(ip) => ip,
            InputResult::Back => return Ok(InputResult::Back),
            InputResult::Cancel => return Ok(InputResult::Cancel),
        };

        match run_text_input(
            stdout,
            music,
            "JOIN GAME",
            "Port",
            "21711",
            5,
            6,
            |c| c.is_ascii_digit(),
            &|s| match s.parse::<u16>() {
                Ok(p) if p > 0 => Ok(()),
                _ => Err("Invalid port".to_string()),
            },
        )? {
            InputResult::Confirm(port) => {
                return Ok(InputResult::Confirm(format!("{}:{}", ip, port)));
            }
            InputResult::Back => continue 'ip,
            InputResult::Cancel => return Ok(InputResult::Cancel),
        }
    }
}
