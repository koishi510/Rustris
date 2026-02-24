mod audio;
mod game;
mod piece;
mod render;
mod settings;

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
use piece::MAX_NEXT_COUNT;
use settings::Settings;

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
    let mut settings = Settings::default();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let result = (|| {
        loop {
            let mode = match select_mode(&mut stdout, &mut music, &mut settings)? {
                Some(m) => m,
                None => return Ok(()),
            };
            match run_game(&mut stdout, mode, &mut music, &mut settings)? {
                true => return Ok(()),
                false => continue,
            }
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}

fn adjust_setting(settings: &mut Settings, sel: usize, direction: i32, mode: GameMode) {
    let mc: usize = match mode {
        GameMode::Marathon => 3,
        GameMode::Sprint | GameMode::Ultra => 1,
    };

    if sel < mc {
        match mode {
            GameMode::Marathon => match sel {
                0 => {
                    let v = settings.level as i32 + direction;
                    settings.level = v.clamp(1, 20) as u32;
                }
                1 => {
                    match (settings.marathon_goal, direction) {
                        (Some(g), 1) if g >= 300 => settings.marathon_goal = None,
                        (Some(g), 1) => settings.marathon_goal = Some((g + 10).min(300)),
                        (Some(g), -1) => settings.marathon_goal = Some(if g <= 10 { 10 } else { g - 10 }),
                        (None, -1) => settings.marathon_goal = Some(300),
                        _ => {}
                    }
                }
                2 => {
                    match (settings.level_cap, direction) {
                        (Some(c), 1) if c >= 20 => settings.level_cap = None,
                        (Some(c), 1) => settings.level_cap = Some((c + 1).min(20)),
                        (Some(c), -1) => settings.level_cap = Some(if c <= 1 { 1 } else { c - 1 }),
                        (None, -1) => settings.level_cap = Some(20),
                        _ => {}
                    }
                }
                _ => {}
            },
            GameMode::Sprint => {
                let v = settings.sprint_goal as i32 + direction * 10;
                settings.sprint_goal = v.clamp(10, 100) as u32;
            }
            GameMode::Ultra => {
                let v = settings.ultra_time as i32 + direction * 10;
                settings.ultra_time = v.clamp(30, 300) as u32;
            }
        }
    } else if sel == mc {
        let v = settings.next_count as i32 + direction;
        settings.next_count = v.clamp(1, MAX_NEXT_COUNT as i32) as usize;
    } else if sel == mc + 1 {
        settings.ghost = !settings.ghost;
    } else if sel == mc + 2 {
        settings.line_clear_anim = !settings.line_clear_anim;
    } else if sel == mc + 3 {
        settings.bag_randomizer = !settings.bag_randomizer;
    }
}

fn run_settings(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
    mode: GameMode,
    in_game: bool,
) -> io::Result<()> {
    let mut sel: usize = 0;

    if in_game {
        let count: usize = 3;
        loop {
            let (bgm_on, sfx_on) = match music.as_ref() {
                Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
                None => (false, false),
            };
            render::draw_settings(stdout, settings, mode, bgm_on, sfx_on, sel, true)?;
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
                            if let Some(m) = music.as_mut() {
                                m.toggle_bgm();
                                m.play_sfx(Sfx::Toggle);
                            }
                        }
                        1 => {
                            if let Some(m) = music.as_mut() {
                                m.toggle_sfx();
                                m.play_sfx(Sfx::Toggle);
                            }
                        }
                        2 => {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuBack);
                            }
                            return Ok(());
                        }
                        _ => {}
                    },
                    KeyCode::Esc => {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuBack);
                        }
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    let mc: usize = match mode {
        GameMode::Marathon => 3,
        GameMode::Sprint | GameMode::Ultra => 1,
    };
    let count = mc + 7;
    let idx_bgm = mc + 4;
    let idx_sfx = mc + 5;
    let idx_back = mc + 6;

    let is_toggle = |s: usize| -> bool {
        s == mc + 1 || s == mc + 2 || s == mc + 3
    };

    loop {
        let (bgm_on, sfx_on) = match music.as_ref() {
            Some(m) => (m.bgm_enabled(), m.sfx_enabled()),
            None => (false, false),
        };
        render::draw_settings(stdout, settings, mode, bgm_on, sfx_on, sel, false)?;
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
                KeyCode::Left => {
                    if sel < idx_bgm {
                        adjust_setting(settings, sel, -1, mode);
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Right => {
                    if sel < idx_bgm {
                        adjust_setting(settings, sel, 1, mode);
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Enter => {
                    if is_toggle(sel) {
                        adjust_setting(settings, sel, 0, mode);
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::Toggle);
                        }
                    } else if sel == idx_bgm {
                        if let Some(m) = music.as_mut() {
                            m.toggle_bgm();
                            m.play_sfx(Sfx::Toggle);
                        }
                    } else if sel == idx_sfx {
                        if let Some(m) = music.as_mut() {
                            m.toggle_sfx();
                            m.play_sfx(Sfx::Toggle);
                        }
                    } else if sel == idx_back {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuBack);
                        }
                        return Ok(());
                    }
                }
                KeyCode::Esc => {
                    if let Some(m) = music.as_ref() {
                        m.play_sfx(Sfx::MenuBack);
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn select_mode(
    stdout: &mut io::Stdout,
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
) -> io::Result<Option<GameMode>> {
    let mut mode = GameMode::Marathon;
    let mut sel: usize = 0;
    let count: usize = 5;

    loop {
        render::draw_mode_select(stdout, mode, sel)?;

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
                KeyCode::Left => {
                    if sel == 0 {
                        mode = match mode {
                            GameMode::Marathon => GameMode::Ultra,
                            GameMode::Sprint => GameMode::Marathon,
                            GameMode::Ultra => GameMode::Sprint,
                        };
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Right => {
                    if sel == 0 {
                        mode = match mode {
                            GameMode::Marathon => GameMode::Sprint,
                            GameMode::Sprint => GameMode::Ultra,
                            GameMode::Ultra => GameMode::Marathon,
                        };
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuMove);
                        }
                    }
                }
                KeyCode::Enter => {
                    if sel == 1 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        return Ok(Some(mode));
                    } else if sel == 2 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        run_settings(stdout, music, settings, mode, false)?;
                    } else if sel == 3 {
                        if let Some(m) = music.as_ref() {
                            m.play_sfx(Sfx::MenuSelect);
                        }
                        render::draw_help(stdout, 0)?;
                        loop {
                            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                if code == KeyCode::Enter || code == KeyCode::Esc {
                                    if let Some(m) = music.as_ref() {
                                        m.play_sfx(Sfx::MenuBack);
                                    }
                                    break;
                                }
                            }
                        }
                    } else if sel == 4 {
                        return Ok(None);
                    }
                }
                _ => {}
            }
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
    music: &mut Option<audio::MusicPlayer>,
    settings: &mut Settings,
) -> io::Result<bool> {
    let mut game = Game::new(mode, settings);
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
            let mut sel: usize = 0;
            let count: usize = 3;
            loop {
                render::draw_game_over(stdout, &game, sel)?;
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
                                break;
                            }
                            1 => {
                                if let Some(m) = music.as_ref() {
                                    m.play_sfx(Sfx::MenuSelect);
                                }
                                return Ok(false);
                            }
                            2 => {
                                return Ok(true);
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            game = Game::new(mode, settings);
            last_tick = Instant::now();
            das = None;
            if let Some(m) = music.as_mut() {
                m.play();
            }
            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
            continue;
        }

        game.update_elapsed();
        if game.mode == GameMode::Ultra && game.elapsed >= Duration::from_secs(game.ultra_time as u64) {
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

        if game.mode == GameMode::Sprint || game.mode == GameMode::Ultra {
            timeout = timeout.min(Duration::from_millis(32));
        }

        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('P') => {
                        if let Some(m) = music.as_mut() {
                            m.play_sfx(Sfx::Pause);
                            m.pause();
                        }
                        let mut sel: usize = 0;
                        let count: usize = 6;
                        let mut retry = false;
                        loop {
                            render::draw_pause(stdout, sel)?;
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
                                            if let Some(m) = music.as_ref() {
                                                m.play_sfx(Sfx::MenuSelect);
                                            }
                                            run_settings(stdout, music, settings, mode, true)?;
                                        }
                                        2 => {
                                            if let Some(m) = music.as_ref() {
                                                m.play_sfx(Sfx::MenuSelect);
                                            }
                                            render::draw_help(stdout, 0)?;
                                            loop {
                                                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                                                    if code == KeyCode::Enter || code == KeyCode::Esc {
                                                        if let Some(m) = music.as_ref() {
                                                            m.play_sfx(Sfx::MenuBack);
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        3 => {
                                            retry = true;
                                            break;
                                        }
                                        4 => {
                                            if let Some(m) = music.as_mut() {
                                                m.play_sfx(Sfx::MenuBack);
                                                m.stop();
                                            }
                                            return Ok(false);
                                        }
                                        5 => {
                                            if let Some(m) = music.as_mut() {
                                                m.stop();
                                            }
                                            return Ok(true);
                                        }
                                        _ => {}
                                    },
                                    KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('P') => {
                                        if let Some(m) = music.as_ref() {
                                            m.play_sfx(Sfx::Resume);
                                        }
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if retry {
                            if let Some(m) = music.as_ref() {
                                m.play_sfx(Sfx::MenuSelect);
                            }
                            game = Game::new(mode, settings);
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
            continue;
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
