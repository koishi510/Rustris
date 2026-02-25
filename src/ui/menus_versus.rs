use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::style::{Color, Stylize};
use crossterm::{cursor, execute};
use std::io;

use crate::audio::{self, Sfx};
use crate::piece::BOARD_WIDTH;
use crate::render;
use crate::settings::Settings;

pub enum VersusAction {
    Host(u16),
    Join(String),
    Back,
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
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    render::draw_title(stdout)?;

    let inner_w = BOARD_WIDTH * 2;

    let label_line = format!("{}:", label);
    let input_line = format!("{}_", input);
    let mut content: Vec<Option<String>> = vec![
        None,
        Some(format!("{:^width$}", title, width = inner_w)),
        None,
        Some(format!("{:^width$}", label_line, width = inner_w)),
        Some(format!("{:^width$}", input_line, width = inner_w)),
        None,
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
    content.push(Some(format!(
        "{:^width$}",
        "Enter / Esc",
        width = inner_w
    )));
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

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    sel = sel.checked_sub(1).unwrap_or(count - 1);
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Down => {
                    sel = (sel + 1) % count;
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Enter => match sel {
                    0 => {
                        // Host
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        match run_port_input(stdout, music)? {
                            Some(port) => return Ok(VersusAction::Host(port)),
                            None => continue,
                        }
                    }
                    1 => {
                        // Join
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        match run_addr_input(stdout, music)? {
                            Some(addr) => return Ok(VersusAction::Join(addr)),
                            None => continue,
                        }
                    }
                    2 => {
                        // Back
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuBack);
                        }
                        return Ok(VersusAction::Back);
                    }
                    _ => {}
                },
                KeyCode::Esc => {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuBack);
                    }
                    return Ok(VersusAction::Back);
                }
                _ => {}
            }
        }
    }
}

fn run_port_input(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<Option<u16>> {
    let mut input = "3000".to_string();
    let mut error = String::new();

    loop {
        draw_input_screen(stdout, "HOST GAME", "Port", &input, &error)?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if input.len() < 5 {
                        input.push(c);
                        error.clear();
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                    error.clear();
                }
                KeyCode::Enter => {
                    match input.parse::<u16>() {
                        Ok(port) if port > 0 => {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
                            return Ok(Some(port));
                        }
                        _ => {
                            error = "Invalid port".to_string();
                        }
                    }
                }
                KeyCode::Esc => {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuBack);
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

fn run_addr_input(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<Option<String>> {
    let mut input = "127.0.0.1:3000".to_string();
    let mut error = String::new();

    loop {
        draw_input_screen(stdout, "JOIN GAME", "Address", &input, &error)?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char(c) if c.is_ascii_graphic() => {
                    if input.len() < 21 {
                        input.push(c);
                        error.clear();
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                    error.clear();
                }
                KeyCode::Enter => {
                    if input.is_empty() {
                        error = "Enter an address".to_string();
                    } else {
                        // Validate basic format
                        if input.contains(':') {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
                            return Ok(Some(input));
                        } else {
                            error = "Use host:port format".to_string();
                        }
                    }
                }
                KeyCode::Esc => {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuBack);
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}
