use crossterm::event::{self, KeyCode};
use crossterm::{execute, terminal};
use std::io;
use std::time::{Duration, Instant};

use crate::audio::{self, Sfx};
use crate::game::garbage::{calculate_attack, GarbageEvent, GarbageQueue};
use crate::game::{Game, GameMode};
use crate::net::transport::Connection;
use crate::net::{BoardSnapshot, GarbageAttack, MatchOutcome, NetMessage, PROTOCOL_VERSION};
use crate::game::piece::*;
use crate::render;
use crate::game::settings::VersusSettings;

use super::input::{self, InputState};
use super::{menu_nav, play_menu_sfx, read_key};

const BOARD_SYNC_INTERVAL: Duration = Duration::from_millis(66);

fn perform_handshake(conn: &mut Connection, is_host: bool) -> io::Result<()> {
    if is_host {
        conn.send(&NetMessage::Hello { version: PROTOCOL_VERSION })?;
        let msg = conn.recv_blocking()?;
        match msg {
            NetMessage::Hello { version } if version == PROTOCOL_VERSION => Ok(()),
            NetMessage::Hello { version } => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("protocol version mismatch: local={}, remote={}", PROTOCOL_VERSION, version),
            )),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected Hello",
            )),
        }
    } else {
        let msg = conn.recv_blocking()?;
        match msg {
            NetMessage::Hello { version } if version == PROTOCOL_VERSION => {
                conn.send(&NetMessage::Hello { version: PROTOCOL_VERSION })?;
                Ok(())
            }
            NetMessage::Hello { version } => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("protocol version mismatch: local={}, remote={}", PROTOCOL_VERSION, version),
            )),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected Hello",
            )),
        }
    }
}

pub fn run_host_lobby(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    port: u16,
) -> io::Result<Option<(Connection, VersusSettings)>> {
    let listener = crate::net::host::listen_nonblocking(port)?;

    let addr_lines: Vec<String> = match crate::net::host::local_ip() {
        Some(ip) => vec![format!("IP: {}", ip), format!("Port: {}", port)],
        None => vec![format!("Port: {}", port)],
    };

    loop {
        let lines: Vec<&str> = std::iter::once("Listening...")
            .chain(addr_lines.iter().map(|s| s.as_str()))
            .collect();
        render::versus::draw_lobby_screen(
            stdout, "HOST GAME", &lines, "", &["Cancel"], 0,
        )?;

        if event::poll(Duration::from_millis(100))? {
            if let Some(code) = read_key()? {
                if code == KeyCode::Enter || code == KeyCode::Esc {
                    play_menu_sfx(music, Sfx::MenuBack);
                    return Ok(None);
                }
            }
        }

        match crate::net::host::try_accept(&listener) {
            Ok(Some(mut conn)) => {
                render::versus::draw_lobby_screen(
                    stdout, "HOST GAME", &["Connected!", "Starting..."], "", &[], 0,
                )?;

                if let Err(e) = perform_handshake(&mut conn, true) {
                    let error_msg = format!("{}", e);
                    let mut sel: usize = 0;
                    let count: usize = 1;
                    loop {
                        render::versus::draw_lobby_screen(
                            stdout, "HOST GAME", &["Handshake failed"], &error_msg, &["Back"], sel,
                        )?;
                        if let Some(code) = read_key()? {
                            match code {
                                KeyCode::Up | KeyCode::Down => {
                                    sel = menu_nav(sel, count, code);
                                    play_menu_sfx(music, Sfx::MenuMove);
                                }
                                KeyCode::Enter | KeyCode::Esc => {
                                    play_menu_sfx(music, Sfx::MenuBack);
                                    return Ok(None);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                let vs = VersusSettings::default();
                conn.send(&NetMessage::LobbySettings(vs))?;

                let msg = conn.recv_blocking()?;
                match msg {
                    NetMessage::Ready => {}
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "expected Ready",
                        ));
                    }
                }

                return Ok(Some((conn, vs)));
            }
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }
}

pub fn run_client_lobby(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    addr: &str,
) -> io::Result<Option<(Connection, VersusSettings)>> {
    let (ip, port) = addr.rsplit_once(':').unwrap_or((addr, ""));
    let addr_lines: Vec<String> = vec![format!("IP: {}", ip), format!("Port: {}", port)];
    let addr_refs: Vec<&str> = addr_lines.iter().map(|s| s.as_str()).collect();

    {
        let lines: Vec<&str> = std::iter::once("Connecting...")
            .chain(addr_refs.iter().copied())
            .collect();
        render::versus::draw_lobby_screen(
            stdout, "JOIN GAME", &lines, "", &[], 0,
        )?;
    }

    let mut conn = match crate::net::client::connect(addr) {
        Ok(c) => c,
        Err(e) => {
            let error_msg = format!("{:?}", e.kind());
            let mut sel: usize = 0;
            let count: usize = 2;
            loop {
                render::versus::draw_lobby_screen(
                    stdout, "JOIN GAME", &addr_refs, &error_msg, &["Retry", "Cancel"], sel,
                )?;
                if let Some(code) = read_key()? {
                    match code {
                        KeyCode::Up | KeyCode::Down => {
                            sel = menu_nav(sel, count, code);
                            play_menu_sfx(music, Sfx::MenuMove);
                        }
                        KeyCode::Enter => match sel {
                            0 => {
                                play_menu_sfx(music, Sfx::MenuSelect);
                                return run_client_lobby(stdout, music, addr);
                            }
                            _ => {
                                play_menu_sfx(music, Sfx::MenuBack);
                                return Ok(None);
                            }
                        },
                        KeyCode::Esc => {
                            play_menu_sfx(music, Sfx::MenuBack);
                            return Ok(None);
                        }
                        _ => {}
                    }
                }
            }
        }
    };

    render::versus::draw_lobby_screen(
        stdout, "JOIN GAME", &["Connected!", "Starting..."], "", &[], 0,
    )?;

    if let Err(e) = perform_handshake(&mut conn, false) {
        let error_msg = format!("{}", e);
        let mut sel: usize = 0;
        let count: usize = 1;
        loop {
            render::versus::draw_lobby_screen(
                stdout, "JOIN GAME", &["Handshake failed"], &error_msg, &["Back"], sel,
            )?;
            if let Some(code) = read_key()? {
                match code {
                    KeyCode::Up | KeyCode::Down => {
                        sel = menu_nav(sel, count, code);
                        play_menu_sfx(music, Sfx::MenuMove);
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        play_menu_sfx(music, Sfx::MenuBack);
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
    }

    let msg = conn.recv_blocking()?;
    let vs = match msg {
        NetMessage::LobbySettings(vs) => vs,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected LobbySettings",
            ));
        }
    };

    conn.send(&NetMessage::Ready)?;

    play_menu_sfx(music, Sfx::MenuSelect);

    Ok(Some((conn, vs)))
}

fn run_countdown(
    stdout: &mut io::Stdout,
    conn: &mut Connection,
    is_host: bool,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<bool> {
    if is_host {
        for count in (1..=3).rev() {
            conn.send(&NetMessage::Countdown(count))?;
            render::versus::draw_versus_countdown(stdout, count)?;
            play_menu_sfx(music, Sfx::MenuMove);
            std::thread::sleep(Duration::from_secs(1));
        }
        conn.send(&NetMessage::GameStart)?;
    } else {
        render::versus::draw_versus_countdown(stdout, 0)?;
        loop {
            let msg = conn.recv_blocking()?;
            match msg {
                NetMessage::Countdown(n) => {
                    render::versus::draw_versus_countdown(stdout, n)?;
                    play_menu_sfx(music, Sfx::MenuMove);
                }
                NetMessage::GameStart => break,
                NetMessage::Disconnect => return Ok(false),
                _ => {}
            }
        }
    }
    Ok(true)
}

pub fn run_versus(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    conn: &mut Connection,
    vs_settings: &VersusSettings,
    is_host: bool,
) -> io::Result<bool> {
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    loop {
        if !run_countdown(stdout, conn, is_host, music)? {
            return Ok(false);
        }

        let game_settings = vs_settings.to_settings();
        let mut game = Game::new(GameMode::Versus, &game_settings);
        let mut garbage_queue = GarbageQueue::new();
        let mut opponent_snapshot: Option<BoardSnapshot> = None;

        let mut inp = InputState::new();
        let mut last_board_sync = Instant::now();
        let mut opponent_dead = false;
        let mut we_died = false;

        if let Some(m) = music.as_mut() {
            m.play();
        }
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        loop {
            if game.game_over && !we_died {
                we_died = true;
                let _ = conn.send(&NetMessage::PlayerDead);
                if let Some(m) = music.as_mut() {
                    m.stop();
                    m.play_sfx(Sfx::GameOver);
                }
            }

            if we_died || opponent_dead {
                break;
            }

            game.update_elapsed();

            render::versus::draw_versus(
                stdout,
                &game,
                &opponent_snapshot,
                garbage_queue.total_pending(),
            )?;

            if last_board_sync.elapsed() >= BOARD_SYNC_INTERVAL {
                let snap = BoardSnapshot::from_game(&game, garbage_queue.total_pending());
                let _ = conn.send(&NetMessage::BoardState(snap));
                last_board_sync = Instant::now();
            }

            loop {
                match conn.try_recv() {
                    Ok(Some(msg)) => match msg {
                        NetMessage::GarbageAttack(ga) => {
                            garbage_queue.push(GarbageEvent {
                                lines: ga.lines,
                                hole_column: ga.hole_column,
                            });
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::GarbageReceived);
                            }
                        }
                        NetMessage::BoardState(snap) => {
                            opponent_snapshot = Some(snap);
                        }
                        NetMessage::PlayerDead => {
                            opponent_dead = true;
                        }
                        NetMessage::MatchResult(_) => {}
                        NetMessage::Disconnect => {
                            if let Some(m) = music.as_mut() {
                                m.stop();
                            }
                            return Ok(false);
                        }
                        _ => {}
                    },
                    Ok(None) => break,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) if e.kind() == io::ErrorKind::ConnectionAborted => {
                        if let Some(m) = music.as_mut() {
                            m.stop();
                        }
                        return Ok(false);
                    }
                    Err(_) => break,
                }
            }

            if opponent_dead {
                break;
            }

            if game.is_animating() {
                if game.update_animation() {
                    if event::poll(Duration::from_millis(16))? {
                        let _ = read_key()?;
                    }
                    continue;
                } else {
                    game.finish_clear();
                    inp.last_tick = Instant::now();
                    continue;
                }
            }

            if game.is_garbage_animating() {
                if game.update_garbage_animation() {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::GarbageReceived);
                    }
                    render::versus::draw_versus(
                        stdout,
                        &game,
                        &opponent_snapshot,
                        garbage_queue.total_pending(),
                    )?;
                    if event::poll(Duration::from_millis(16))? {
                        let _ = read_key()?;
                    }
                    continue;
                }
                inp.last_tick = Instant::now();
            }

            let mut timeout = input::compute_timeout(&game, &inp);
            timeout = timeout.min(Duration::from_millis(16));

            if event::poll(timeout)? {
                if let Some(code) = read_key()? {
                    match code {
                        KeyCode::Esc => {
                            if let Some(m) = music.as_mut() {
                                m.play_sfx(Sfx::Pause);
                            }
                            let mut sel: usize = 0;
                            let count: usize = 2;
                            let mut forfeit = false;
                            loop {
                                render::versus::draw_versus_forfeit(stdout, sel)?;

                                if let Some(code) = read_key()? {
                                    match code {
                                        KeyCode::Up | KeyCode::Down => {
                                            sel = menu_nav(sel, count, code);
                                            play_menu_sfx(music, Sfx::MenuMove);
                                        }
                                        KeyCode::Enter => match sel {
                                            0 => {
                                                if let Some(m) = music.as_ref() {
                                                    m.play_sfx(Sfx::Resume);
                                                }
                                                break;
                                            }
                                            1 => {
                                                forfeit = true;
                                                break;
                                            }
                                            _ => {}
                                        },
                                        KeyCode::Esc => {
                                            if let Some(m) = music.as_ref() {
                                                m.play_sfx(Sfx::Resume);
                                            }
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            if forfeit {
                                game.game_over = true;
                                continue;
                            }
                            if game.lock_delay.is_some() {
                                game.lock_delay = Some(Instant::now());
                            }
                            inp.last_tick = Instant::now();
                            if let Some(d) = &mut inp.das {
                                let now = Instant::now();
                                d.last_event = now;
                                d.start = now;
                                d.last_arr_move = now;
                            }
                            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                            continue;
                        }
                        other => {
                            let hard_dropped = input::handle_game_key(other, &mut game, &mut inp, music);
                            if hard_dropped {
                                process_post_lock(
                                    &mut game,
                                    &mut garbage_queue,
                                    conn,
                                    music,
                                )?;
                            }
                        }
                    }
                }
            }

            if game.in_are() {
                input::update_game_timers(&mut game, &mut inp, music);
                continue;
            }

            let locked = input::update_game_timers(&mut game, &mut inp, music);
            if locked {
                process_post_lock(&mut game, &mut garbage_queue, conn, music)?;
            }
        }

        if let Some(m) = music.as_mut() {
            m.stop();
        }

        let won = if is_host {
            if we_died && !opponent_dead {
                let _ = conn.send(&NetMessage::MatchResult(MatchOutcome::Lose));
                false
            } else {
                let _ = conn.send(&NetMessage::MatchResult(MatchOutcome::Win));
                true
            }
        } else {
            opponent_dead && !we_died
        };

        if won {
            if let Some(m) = music.as_ref() {
                m.play_sfx(Sfx::VersusWin);
            }
        } else if let Some(m) = music.as_ref() {
            m.play_sfx(Sfx::VersusLose);
        }

        let rematch = run_result_screen(stdout, music, conn, won)?;

        match rematch {
            ResultAction::Rematch => {
                continue;
            }
            ResultAction::Menu => return Ok(false),
            ResultAction::Quit => return Ok(true),
        }
    }
}

fn process_post_lock(
    game: &mut Game,
    garbage_queue: &mut GarbageQueue,
    conn: &mut Connection,
    music: &Option<audio::MusicPlayer>,
) -> io::Result<()> {
    if let Some(action) = &game.last_action {
        if action.cleared_lines > 0 {
            let attack = calculate_attack(action);
            if attack > 0 {
                let remaining = garbage_queue.cancel(attack);
                if remaining > 0 {
                    use rand::Rng;
                    let hole = rand::thread_rng().gen_range(0..BOARD_WIDTH);
                    let _ = conn.send(&NetMessage::GarbageAttack(GarbageAttack {
                        lines: remaining,
                        hole_column: hole,
                    }));
                }
            }
        } else {
            apply_pending_garbage(game, garbage_queue, music);
        }
    } else {
        apply_pending_garbage(game, garbage_queue, music);
    }
    Ok(())
}

fn apply_pending_garbage(
    game: &mut Game,
    garbage_queue: &mut GarbageQueue,
    _music: &Option<audio::MusicPlayer>,
) {
    let events = garbage_queue.drain_all();
    if events.is_empty() {
        return;
    }
    game.begin_garbage_rise(events);
}

enum ResultAction {
    Rematch,
    Menu,
    Quit,
}

fn run_result_screen(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    conn: &mut Connection,
    won: bool,
) -> io::Result<ResultAction> {
    let mut sel: usize = 0;
    let count: usize = 3;
    let mut _we_requested_rematch = false;
    let mut opponent_requested_rematch = false;

    loop {
        render::versus::draw_versus_result(stdout, won, sel)?;

        match conn.try_recv() {
            Ok(Some(NetMessage::RematchRequest)) => {
                opponent_requested_rematch = true;
                if _we_requested_rematch {
                    let _ = conn.send(&NetMessage::RematchAccept);
                    return Ok(ResultAction::Rematch);
                }
            }
            Ok(Some(NetMessage::RematchAccept)) => {
                return Ok(ResultAction::Rematch);
            }
            Ok(Some(NetMessage::Disconnect)) => {
                return Ok(ResultAction::Menu);
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::ConnectionAborted => {
                return Ok(ResultAction::Menu);
            }
            Err(_) => {}
        }

        if event::poll(Duration::from_millis(50))? {
            if let Some(code) = read_key()? {
                match code {
                    KeyCode::Up | KeyCode::Down => {
                        sel = menu_nav(sel, count, code);
                        play_menu_sfx(music, Sfx::MenuMove);
                    }
                    KeyCode::Enter => match sel {
                        0 => {
                            play_menu_sfx(music, Sfx::MenuSelect);
                            let _ = conn.send(&NetMessage::RematchRequest);
                            _we_requested_rematch = true;

                            if opponent_requested_rematch {
                                let _ = conn.send(&NetMessage::RematchAccept);
                                return Ok(ResultAction::Rematch);
                            }

                            render::versus::draw_versus_waiting_rematch(stdout)?;
                            loop {
                                match conn.try_recv() {
                                    Ok(Some(NetMessage::RematchRequest)) => {
                                        let _ = conn.send(&NetMessage::RematchAccept);
                                        return Ok(ResultAction::Rematch);
                                    }
                                    Ok(Some(NetMessage::RematchAccept)) => {
                                        return Ok(ResultAction::Rematch);
                                    }
                                    Ok(Some(NetMessage::Disconnect)) => {
                                        return Ok(ResultAction::Menu);
                                    }
                                    Ok(_) => {}
                                    Err(e) if e.kind() == io::ErrorKind::ConnectionAborted => {
                                        return Ok(ResultAction::Menu);
                                    }
                                    Err(_) => {}
                                }

                                if event::poll(Duration::from_millis(50))? {
                                    if let Some(code) = read_key()? {
                                        if code == KeyCode::Esc || code == KeyCode::Enter {
                                            play_menu_sfx(music, Sfx::MenuBack);
                                            let _ = conn.send(&NetMessage::Disconnect);
                                            return Ok(ResultAction::Menu);
                                        }
                                    }
                                }
                            }
                        }
                        1 => {
                            play_menu_sfx(music, Sfx::MenuBack);
                            let _ = conn.send(&NetMessage::Disconnect);
                            return Ok(ResultAction::Menu);
                        }
                        2 => {
                            let _ = conn.send(&NetMessage::Disconnect);
                            return Ok(ResultAction::Quit);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}
