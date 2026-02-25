use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::{execute, terminal};
use std::io;
use std::time::{Duration, Instant};

use crate::audio::{self, Sfx};
use crate::game::garbage::{calculate_attack, GarbageEvent, GarbageQueue};
use crate::game::{Game, GameMode, LastMove, ARE_DELAY};
use crate::net::transport::Connection;
use crate::net::{BoardSnapshot, GarbageAttack, MatchOutcome, NetMessage};
use crate::piece::*;
use crate::render;
use crate::settings::{Settings, VersusSettings};

const DAS_DELAY: Duration = Duration::from_millis(167);
const ARR_INTERVAL: Duration = Duration::from_millis(33);
const DAS_RELEASE: Duration = Duration::from_millis(100);
const BOARD_SYNC_INTERVAL: Duration = Duration::from_millis(66);

struct DasState {
    direction: i32,
    start: Instant,
    charged: bool,
    last_arr_move: Instant,
    last_event: Instant,
}

impl DasState {
    fn new(direction: i32) -> Self {
        let now = Instant::now();
        Self {
            direction,
            start: now,
            charged: false,
            last_arr_move: now,
            last_event: now,
        }
    }
}

fn make_board_snapshot(game: &Game, pending_garbage: u32) -> BoardSnapshot {
    let mut board = Vec::with_capacity(BOARD_WIDTH * BOARD_HEIGHT);
    for row in 0..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            board.push(game.board[row][col]);
        }
    }

    let current_cells = if game.is_animating() || game.in_are() {
        vec![]
    } else {
        game.current.cells().to_vec()
    };

    let current_kind = game.current.kind;

    BoardSnapshot {
        board,
        current_cells,
        current_kind,
        score: game.score,
        lines: game.lines,
        pending_garbage,
    }
}

fn play_clear_sfx(music: &audio::MusicPlayer, game: &Game, prev_level: u32) {
    music.play_sfx(Sfx::Lock);

    if let Some(anim) = &game.line_clear_anim {
        let lines = anim.rows.len() as u32;
        if let Some(action) = &game.last_action {
            let label = &action.label;
            if label.contains("T-Spin") || label.contains("Mini T-Spin") {
                music.play_sfx(Sfx::TSpinClear(lines));
            } else {
                music.play_sfx(Sfx::LineClear(lines));
            }
            if label.contains("Combo") {
                music.play_sfx(Sfx::Combo(game.combo as u32));
            }
            if label.contains("B2B") {
                music.play_sfx(Sfx::BackToBack);
            }
            if label.contains("ALL CLEAR") {
                music.play_sfx(Sfx::AllClear);
            }
        }
    } else if !game.game_over {
        if let Some(action) = &game.last_action {
            if game.last_action_time.elapsed().as_millis() < 100 {
                if action.label.contains("Mini T-Spin") {
                    music.play_sfx(Sfx::TSpinMini);
                } else if action.label.contains("T-Spin") {
                    music.play_sfx(Sfx::TSpin);
                }
            }
        }
    }

    if game.level > prev_level {
        music.play_sfx(Sfx::LevelUp);
    }
}

/// Host lobby: wait for client connection
pub fn run_host_lobby(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    settings: &Settings,
    port: u16,
) -> io::Result<Option<(Connection, VersusSettings)>> {
    let listener = crate::net::host::listen_nonblocking(port)?;

    let port_str = port.to_string();
    render::versus::draw_versus_lobby(stdout, true, &["Listening...", &format!("Port: {}", port_str)])?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                if code == KeyCode::Esc {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuBack);
                    }
                    return Ok(None);
                }
            }
        }

        match crate::net::host::try_accept(&listener) {
            Ok(Some(mut conn)) => {
                let vs = VersusSettings::from_settings(settings);
                conn.send(&NetMessage::LobbySettings(vs))?;

                render::versus::draw_versus_lobby(
                    stdout,
                    true,
                    &["Connected!", "Waiting..."],
                )?;

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
    render::versus::draw_versus_lobby(stdout, false, &["Connecting...", addr])?;

    let mut conn = match crate::net::client::connect(addr) {
        Ok(c) => c,
        Err(_) => {
            render::versus::draw_versus_lobby(stdout, false, &["Connect failed!", "Check address"])?;
            loop {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    if code == KeyCode::Enter || code == KeyCode::Esc {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuBack);
                        }
                        return Ok(None);
                    }
                }
            }
        }
    };

    render::versus::draw_versus_lobby(stdout, false, &["Connected!", "Waiting..."])?;

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

    if let Some(m) = music.as_ref() {
        m.play_sfx(Sfx::MenuSelect);
    }

    Ok(Some((conn, vs)))
}

/// Run the countdown sequence
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
            if let Some(m) = music.as_ref() {
                m.play_sfx(Sfx::MenuMove);
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        conn.send(&NetMessage::GameStart)?;
    } else {
        // Show waiting screen while host hasn't sent countdown yet
        render::versus::draw_versus_countdown(stdout, 0)?;
        loop {
            let msg = conn.recv_blocking()?;
            match msg {
                NetMessage::Countdown(n) => {
                    render::versus::draw_versus_countdown(stdout, n)?;
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                NetMessage::GameStart => break,
                NetMessage::Disconnect => return Ok(false),
                _ => {}
            }
        }
    }
    Ok(true)
}

/// Main versus game loop
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

        let mut last_tick = Instant::now();
        let mut last_board_sync = Instant::now();
        let mut das: Option<DasState> = None;
        let mut irs: Option<i32> = None;
        let mut ihs: bool = false;
        let mut opponent_dead = false;
        let mut we_died = false;

        if let Some(m) = music.as_mut() {
            m.play();
        }
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        let play_move_sfx = |music: &Option<audio::MusicPlayer>| {
            if let Some(m) = music.as_ref() {
                m.play_sfx(Sfx::Move);
            }
        };

        loop {
            if game.game_over && !we_died {
                we_died = true;
                let _ = conn.send(&NetMessage::PlayerDead);
                if let Some(m) = music.as_mut() {
                    m.stop();
                    m.play_sfx(Sfx::GameOver);
                }
            }

            if we_died && opponent_dead {
                break;
            }

            if we_died && !opponent_dead {
                break;
            }

            if opponent_dead && !we_died {
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
                let snap = make_board_snapshot(&game, garbage_queue.total_pending());
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
                        if let Event::Key(_) = event::read()? {}
                    }
                    continue;
                } else {
                    game.finish_clear();
                    last_tick = Instant::now();
                    continue;
                }
            }

            let mut timeout = if game.in_are() {
                Duration::from_secs(1)
            } else {
                let gravity_remaining = game.drop_interval().saturating_sub(last_tick.elapsed());
                if let Some(lock_start) = game.lock_delay {
                    let lock_remaining =
                        game.lock_delay_duration().saturating_sub(lock_start.elapsed());
                    gravity_remaining.min(lock_remaining)
                } else {
                    gravity_remaining
                }
            };

            if let Some(are_start) = game.are_timer {
                timeout = timeout.min(ARE_DELAY.saturating_sub(are_start.elapsed()));
            }

            if let Some(d) = &das {
                timeout = timeout.min(DAS_RELEASE.saturating_sub(d.last_event.elapsed()));
                if !d.charged {
                    timeout = timeout.min(DAS_DELAY.saturating_sub(d.start.elapsed()));
                } else {
                    timeout = timeout.min(ARR_INTERVAL.saturating_sub(d.last_arr_move.elapsed()));
                }
            }

            timeout = timeout.min(Duration::from_millis(16));

            if event::poll(timeout)? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
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
                            last_tick = Instant::now();
                            if let Some(d) = &mut das {
                                let now = Instant::now();
                                d.last_event = now;
                                d.start = now;
                                d.last_arr_move = now;
                            }
                            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                            continue;
                        }
                        KeyCode::Left | KeyCode::Right => {
                            let dir = if code == KeyCode::Left { -1 } else { 1 };
                            if das.as_ref().map_or(false, |d| d.direction == dir) {
                                das.as_mut().unwrap().last_event = Instant::now();
                            } else {
                                das = Some(DasState::new(dir));
                                if !game.in_are() {
                                    if game.move_piece(0, dir) {
                                        play_move_sfx(&music);
                                    }
                                }
                            }
                        }
                        KeyCode::Down => {
                            if !game.in_are() {
                                game.soft_drop();
                            }
                        }
                        KeyCode::Up | KeyCode::Char('x') | KeyCode::Char('X') => {
                            if game.in_are() {
                                irs = Some(1);
                            } else {
                                game.rotate_cw();
                                if game.last_move == LastMove::Rotate {
                                    if let Some(m) = music.as_ref() {
                                        m.play_sfx(Sfx::Rotate);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('z') | KeyCode::Char('Z') => {
                            if game.in_are() {
                                irs = Some(-1);
                            } else {
                                game.rotate_ccw();
                                if game.last_move == LastMove::Rotate {
                                    if let Some(m) = music.as_ref() {
                                        m.play_sfx(Sfx::Rotate);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if game.in_are() {
                                ihs = true;
                            } else {
                                let was_used = game.hold_used;
                                game.hold_piece();
                                if !was_used && game.hold_used {
                                    if let Some(m) = music.as_ref() {
                                        m.play_sfx(Sfx::Hold);
                                    }
                                }
                            }
                        }
                        KeyCode::Char(' ') => {
                            if !game.in_are() {
                                let prev_level = game.level;
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::HardDrop);
                                }
                                game.hard_drop();
                                if let Some(m) = music.as_ref() {
                                    play_clear_sfx(m, &game, prev_level);
                                }

                                process_post_lock(
                                    &mut game,
                                    &mut garbage_queue,
                                    conn,
                                    music,
                                )?;

                                last_tick = Instant::now();
                            }
                        }
                        _ => {}
                    }
                }
            }

            if let Some(d) = &das {
                if d.last_event.elapsed() >= DAS_RELEASE {
                    das = None;
                }
            }

            if let Some(d) = &mut das {
                if !game.in_are() {
                    if !d.charged && d.start.elapsed() >= DAS_DELAY {
                        d.charged = true;
                        d.last_arr_move = Instant::now();
                        if game.move_piece(0, d.direction) {
                            play_move_sfx(&music);
                        }
                    } else if d.charged && d.last_arr_move.elapsed() >= ARR_INTERVAL {
                        d.last_arr_move = Instant::now();
                        if game.move_piece(0, d.direction) {
                            play_move_sfx(&music);
                        }
                    }
                }
            }

            if game.in_are() {
                if game.check_are() {
                    if ihs {
                        ihs = false;
                        let was_used = game.hold_used;
                        game.hold_piece();
                        if !was_used && game.hold_used {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::Hold);
                            }
                        }
                    }
                    if let Some(dir) = irs.take() {
                        if dir > 0 {
                            game.rotate_cw();
                        } else {
                            game.rotate_ccw();
                        }
                        if game.last_move == LastMove::Rotate {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::Rotate);
                            }
                        }
                    }
                    last_tick = Instant::now();
                    if let Some(d) = &mut das {
                        if d.charged {
                            while game.move_piece(0, d.direction) {}
                            d.last_arr_move = Instant::now();
                        }
                    }
                }
                continue;
            }

            if let Some(lock_start) = game.lock_delay {
                if lock_start.elapsed() >= game.lock_delay_duration() {
                    game.lock_delay = None;
                    let prev_level = game.level;
                    game.lock_and_begin_clear();
                    if let Some(m) = music.as_ref() {
                        play_clear_sfx(m, &game, prev_level);
                    }

                    process_post_lock(&mut game, &mut garbage_queue, conn, music)?;

                    last_tick = Instant::now();
                    continue;
                }
            }

            if last_tick.elapsed() >= game.drop_interval() {
                game.tick();
                last_tick = Instant::now();
            }
        }

        if let Some(m) = music.as_mut() {
            m.stop();
        }

        let won = if is_host {
            if we_died && !opponent_dead {
                let _ = conn.send(&NetMessage::MatchResult(MatchOutcome::Lose));
                false
            } else if opponent_dead && !we_died {
                let _ = conn.send(&NetMessage::MatchResult(MatchOutcome::Win));
                true
            } else {
                let _ = conn.send(&NetMessage::MatchResult(MatchOutcome::Win));
                true
            }
        } else {
            if we_died && !opponent_dead {
                false
            } else if opponent_dead && !we_died {
                true
            } else {
                false
            }
        };

        if won {
            if let Some(m) = music.as_ref() {
                m.play_sfx(Sfx::VersusWin);
            }
        } else {
            if let Some(m) = music.as_ref() {
                m.play_sfx(Sfx::VersusLose);
            }
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
    music: &Option<audio::MusicPlayer>,
) {
    let events = garbage_queue.drain_all();
    let had_garbage = !events.is_empty();
    for event in events {
        game.receive_garbage(event.lines, event.hole_column);
    }
    if had_garbage {
        if let Some(m) = music.as_ref() {
            m.play_sfx(Sfx::GarbageReceived);
        }
    }
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
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
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
                                    if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                        if code == KeyCode::Esc {
                                            let _ = conn.send(&NetMessage::Disconnect);
                                            return Ok(ResultAction::Menu);
                                        }
                                    }
                                }
                            }
                        }
                        1 => {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuBack);
                            }
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
