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
use game::{Game, LastMove, LOCK_DELAY};

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
            let level = match select_level(&mut stdout, &mut music)? {
                Some(l) => l,
                None => return Ok(()),
            };
            run_game(&mut stdout, level, &mut music)?;
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}

fn select_level(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<Option<u32>> {
    let mut level: u32 = 1;

    let (bgm_on, sfx_on) = match music {
        Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
        None => (false, false),
    };

    render::draw_empty_board(stdout)?;
    render::draw_level_select(stdout, level, bgm_on, sfx_on)?;

    loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    if level < 25 {
                        level += 1;
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Down => {
                    if level > 1 {
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
                    return Ok(Some(level));
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
            render::draw_empty_board(stdout)?;
            render::draw_level_select(stdout, level, bgm_on, sfx_on)?;
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
    start_level: u32,
    music: &mut Option<audio::MusicPlayer>,
) -> io::Result<()> {
    let mut game = Game::new(start_level);
    let mut last_tick = Instant::now();
    if let Some(m) = music.as_mut() {
        m.play();
    }
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    loop {
        if game.game_over {
            if let Some(m) = music.as_mut() {
                m.stop();
                m.play_sfx(Sfx::GameOver);
            }
            render::draw(stdout, &game)?;
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
            game = Game::new(start_level);
            last_tick = Instant::now();
            if let Some(m) = music.as_mut() {
                m.play();
            }
            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
            continue;
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

        let gravity_remaining = game.drop_interval().saturating_sub(last_tick.elapsed());
        let timeout = if let Some(lock_start) = game.lock_delay {
            let lock_remaining = LOCK_DELAY.saturating_sub(lock_start.elapsed());
            gravity_remaining.min(lock_remaining)
        } else {
            gravity_remaining
        };

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
                                    _ => {}
                                }
                            }
                        }
                        if let Some(m) = music.as_mut() {
                            m.resume();
                            m.play_sfx(Sfx::Resume);
                        }
                        last_tick = Instant::now();
                        if game.lock_delay.is_some() {
                            game.lock_delay = Some(Instant::now());
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
                    KeyCode::Left => {
                        if game.move_piece(0, -1) {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::Move);
                            }
                        }
                    }
                    KeyCode::Right => {
                        if game.move_piece(0, 1) {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::Move);
                            }
                        }
                    }
                    KeyCode::Down => {
                        game.soft_drop();
                    }
                    KeyCode::Up | KeyCode::Char('x') | KeyCode::Char('X') => {
                        game.rotate_cw();
                        if game.last_move == LastMove::Rotate {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::Rotate);
                            }
                        }
                    }
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        game.rotate_ccw();
                        if game.last_move == LastMove::Rotate {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::Rotate);
                            }
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        let was_used = game.hold_used;
                        game.hold_piece();
                        if !was_used {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::Hold);
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
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
                    _ => {}
                }
            }
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
