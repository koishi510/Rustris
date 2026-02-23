mod audio;
mod game;
mod piece;
mod render;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal,
};
use std::io;
use std::time::{Duration, Instant};

use audio::Sfx;
use game::{Game, GameMode, LastMove, ARE_DELAY, LOCK_DELAY};

const DAS_DELAY: Duration = Duration::from_millis(167);
const ARR_INTERVAL: Duration = Duration::from_millis(33);
const DAS_RELEASE: Duration = Duration::from_millis(100);

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

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let mut music = audio::MusicPlayer::new();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let result = (|| {
        loop {
            let (mode, level) = match select_mode(&mut stdout, &mut music)? {
                Some(ml) => ml,
                None => return Ok(()),
            };
            run_game(&mut stdout, mode, level, &mut music)?;
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}

fn select_mode(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<Option<(GameMode, u32)>> {
    let mut mode = GameMode::Marathon;
    let mut level: u32 = 1;

    let (bgm_on, sfx_on) = match music {
        Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
        None => (false, false),
    };

    render::draw_mode_select(stdout, mode, level, bgm_on, sfx_on)?;

    loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Left => {
                    mode = match mode {
                        GameMode::Marathon => GameMode::Endless,
                        GameMode::Sprint => GameMode::Marathon,
                        GameMode::Ultra => GameMode::Sprint,
                        GameMode::Endless => GameMode::Ultra,
                    };
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Right => {
                    mode = match mode {
                        GameMode::Marathon => GameMode::Sprint,
                        GameMode::Sprint => GameMode::Ultra,
                        GameMode::Ultra => GameMode::Endless,
                        GameMode::Endless => GameMode::Marathon,
                    };
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuMove);
                    }
                }
                KeyCode::Up => {
                    if (mode == GameMode::Marathon || mode == GameMode::Endless) && level < 20 {
                        level += 1;
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Down => {
                    if (mode == GameMode::Marathon || mode == GameMode::Endless) && level > 1 {
                        level -= 1;
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuSelect);
                    }
                    let start_level = match mode {
                        GameMode::Marathon | GameMode::Endless => level,
                        GameMode::Sprint | GameMode::Ultra => 1,
                    };
                    return Ok(Some((mode, start_level)));
                }
                KeyCode::Char('h') | KeyCode::Char('H') => {
                    render::draw_help(stdout)?;
                    loop {
                        if let Event::Key(KeyEvent { code: KeyCode::Esc, .. }) = event::read()? {
                            break;
                        }
                    }
                }
                KeyCode::Char('m') | KeyCode::Char('M') => {
                    if let Some(m) = music.as_mut() {
                        m.toggle_bgm();
                    }
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if let Some(m) = music.as_mut() {
                        m.toggle_sfx();
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(None),
                _ => continue,
            }

            let (bgm_on, sfx_on) = match music {
                Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
                None => (false, false),
            };
            render::draw_mode_select(stdout, mode, level, bgm_on, sfx_on)?;
        }
    }
}

fn play_clear_sfx(music: &audio::MusicPlayer, game: &Game, prev_level: u32) {
    music.play_sfx(Sfx::Lock);

    if let Some(anim) = &game.line_clear_anim {
        let lines = anim.rows.len() as u32;
        if let Some(action) = &game.last_action {
            let label = &action.label;
            // Priority from low to high (last call wins)
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

fn run_game(
    stdout: &mut io::Stdout,
    mode: GameMode,
    start_level: u32,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<()> {
    let mut game = Game::new(start_level, mode);
    let mut last_tick = Instant::now();
    let mut das: Option<DasState> = None;
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
        if game.game_over {
            if let Some(m) = music.as_mut() {
                m.stop();
                if game.cleared {
                    m.play_sfx(Sfx::Clear);
                } else {
                    m.play_sfx(Sfx::GameOver);
                }
            }
            render::draw_game_over(stdout, &game)?;
            loop {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
                            return Ok(());
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            }
            game = Game::new(start_level, mode);
            last_tick = Instant::now();
            das = None;
            if let Some(m) = music.as_mut() {
                m.play();
            }
            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
            continue;
        }

        game.update_elapsed();
        if game.mode == GameMode::Ultra && game.elapsed >= Duration::from_secs(120) {
            game.game_over = true;
        }

        render::draw(stdout, &game)?;

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
                let lock_remaining = LOCK_DELAY.saturating_sub(lock_start.elapsed());
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

        if let Some(remaining) = game.time_remaining() {
            timeout = timeout.min(remaining);
        }

        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc => {
                        if let Some(m) = music.as_mut() {
                            m.play_sfx(Sfx::Pause);
                            m.pause();
                        }
                        let (bgm_on, sfx_on) = match music.as_ref() {
                            Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
                            None => (false, false),
                        };
                        render::draw_pause(stdout, bgm_on, sfx_on)?;
                        let mut retry = false;
                        loop {
                            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                match code {
                                    KeyCode::Esc => break,
                                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        if let Some(m) = music.as_mut() {
                                            m.stop();
                                        }
                                        return Ok(());
                                    }
                                    KeyCode::Char('r') | KeyCode::Char('R') => {
                                        retry = true;
                                        break;
                                    }
                                    KeyCode::Char('m') | KeyCode::Char('M') => {
                                        if let Some(m) = music.as_mut() {
                                            m.toggle_bgm();
                                        }
                                        let (bgm_on, sfx_on) = match music.as_ref() {
                                            Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
                                            None => (false, false),
                                        };
                                        render::draw_pause(stdout, bgm_on, sfx_on)?;
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') => {
                                        if let Some(m) = music.as_mut() {
                                            m.toggle_sfx();
                                        }
                                        let (bgm_on, sfx_on) = match music.as_ref() {
                                            Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
                                            None => (false, false),
                                        };
                                        render::draw_pause(stdout, bgm_on, sfx_on)?;
                                    }
                                    KeyCode::Char('h') | KeyCode::Char('H') => {
                                        render::draw_help(stdout)?;
                                        loop {
                                            if let Event::Key(KeyEvent { code: KeyCode::Esc, .. }) = event::read()? {
                                                break;
                                            }
                                        }
                                        let (bgm_on, sfx_on) = match music.as_ref() {
                                            Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
                                            None => (false, false),
                                        };
                                        render::draw_pause(stdout, bgm_on, sfx_on)?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if retry {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
                            game = Game::new(start_level, mode);
                            last_tick = Instant::now();
                            das = None;
                            if let Some(m) = music.as_mut() {
                                m.play();
                            }
                            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                            continue;
                        }
                        if let Some(m) = music.as_mut() {
                            m.resume();
                            m.play_sfx(Sfx::Resume);
                        }
                        game.reset_game_start();
                        last_tick = Instant::now();
                        if game.lock_delay.is_some() {
                            game.lock_delay = Some(Instant::now());
                        }
                        if let Some(d) = &mut das {
                            let now = Instant::now();
                            d.last_event = now;
                            d.start = now;
                            d.last_arr_move = now;
                        }
                        continue;
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        if let Some(m) = music.as_mut() {
                            m.toggle_bgm();
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        if let Some(m) = music.as_mut() {
                            m.toggle_sfx();
                        }
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
                        if !game.in_are() {
                            game.rotate_cw();
                            if game.last_move == LastMove::Rotate {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::Rotate);
                                }
                            }
                        }
                    }
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        if !game.in_are() {
                            game.rotate_ccw();
                            if game.last_move == LastMove::Rotate {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::Rotate);
                                }
                            }
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if !game.in_are() {
                            let was_used = game.hold_used;
                            game.hold_piece();
                            if !was_used {
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
                last_tick = Instant::now();
                if let Some(d) = &mut das {
                    if d.charged {
                        while game.move_piece(0, d.direction) {}
                        d.last_arr_move = Instant::now();
                    }
                }
            }
            continue; // Skip gravity/lock during ARE
        }

        if let Some(lock_start) = game.lock_delay {
            if lock_start.elapsed() >= LOCK_DELAY {
                game.lock_delay = None;
                let prev_level = game.level;
                game.lock_and_begin_clear();
                if let Some(m) = music.as_ref() {
                    play_clear_sfx(m, &game, prev_level);
                }
                last_tick = Instant::now();
                continue;
            }
        }

        if last_tick.elapsed() >= game.drop_interval() {
            game.tick();
            last_tick = Instant::now();
        }
    }
}
