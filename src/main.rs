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

use game::{Game, LOCK_DELAY};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let result = (|| {
        loop {
            let level = match select_level(&mut stdout)? {
                Some(l) => l,
                None => return Ok(()),
            };
            run_game(&mut stdout, level)?;
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}

fn select_level(stdout: &mut io::Stdout) -> io::Result<Option<u32>> {
    let mut level: u32 = 1;

    render::draw_empty_board(stdout)?;
    render::draw_level_select(stdout, level)?;

    loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    if level < 25 {
                        level += 1;
                    }
                }
                KeyCode::Down => {
                    if level > 1 {
                        level -= 1;
                    }
                }
                KeyCode::Enter => return Ok(Some(level)),
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(None),
                _ => continue,
            }
            render::draw_empty_board(stdout)?;
            render::draw_level_select(stdout, level)?;
        }
    }
}

fn run_game(stdout: &mut io::Stdout, start_level: u32) -> io::Result<()> {
    let mut game = Game::new(start_level);
    let mut last_tick = Instant::now();
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    loop {
        if game.game_over {
            render::draw(stdout, &game)?;
            render::draw_game_over(stdout, &game)?;
            loop {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }

        render::draw(stdout, &game)?;

        if game.is_animating() {
            if game.update_animation() {
                if event::poll(Duration::from_millis(16))? {
                    if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                        if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                            return Ok(());
                        }
                    }
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
                        render::draw_pause(stdout)?;
                        loop {
                            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                match code {
                                    KeyCode::Esc => break,
                                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                                    _ => {}
                                }
                            }
                        }
                        // Reset timers after unpause
                        last_tick = Instant::now();
                        if game.lock_delay.is_some() {
                            game.lock_delay = Some(Instant::now());
                        }
                        continue;
                    }
                    KeyCode::Left => {
                        game.move_piece(0, -1);
                    }
                    KeyCode::Right => {
                        game.move_piece(0, 1);
                    }
                    KeyCode::Down => {
                        game.soft_drop();
                    }
                    KeyCode::Up | KeyCode::Char('x') | KeyCode::Char('X') => {
                        game.rotate_cw();
                    }
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        game.rotate_ccw();
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        game.hold_piece();
                    }
                    KeyCode::Char(' ') => {
                        game.hard_drop();
                        last_tick = Instant::now();
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    _ => {}
                }
            }
        }

        if let Some(lock_start) = game.lock_delay {
            if lock_start.elapsed() >= LOCK_DELAY {
                game.lock_delay = None;
                game.lock_and_begin_clear();
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
