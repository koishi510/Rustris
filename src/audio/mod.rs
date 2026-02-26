use rodio::source::Source;
use rodio::{OutputStream, Sink};
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

#[cfg(unix)]
fn suppress_stderr() -> RawFd {
    let devnull = std::fs::File::open("/dev/null").ok();
    let saved = unsafe { libc_dup(2) };
    if let Some(f) = devnull {
        unsafe { libc_dup2(f.as_raw_fd(), 2) };
    }
    saved
}

#[cfg(unix)]
fn restore_stderr(saved: RawFd) {
    if saved >= 0 {
        unsafe {
            libc_dup2(saved, 2);
            libc_close(saved);
        }
    }
}

#[cfg(unix)]
unsafe fn libc_dup(fd: RawFd) -> RawFd {
    unsafe extern "C" { fn dup(fd: i32) -> i32; }
    unsafe { dup(fd) }
}
#[cfg(unix)]
unsafe fn libc_dup2(oldfd: RawFd, newfd: RawFd) -> RawFd {
    unsafe extern "C" { fn dup2(oldfd: i32, newfd: i32) -> i32; }
    unsafe { dup2(oldfd, newfd) }
}
#[cfg(unix)]
unsafe fn libc_close(fd: RawFd) -> i32 {
    unsafe extern "C" { fn close(fd: i32) -> i32; }
    unsafe { close(fd) }
}

#[cfg(unix)]
fn init_output_stream() -> Option<(OutputStream, rodio::OutputStreamHandle)> {
    let saved = suppress_stderr();
    let result = OutputStream::try_default().ok();
    restore_stderr(saved);
    result
}

#[cfg(not(unix))]
fn init_output_stream() -> Option<(OutputStream, rodio::OutputStreamHandle)> {
    OutputStream::try_default().ok()
}

const SAMPLE_RATE: u32 = 44100;
const BPM: f32 = 140.0;
const BEAT_DURATION: f32 = 60.0 / BPM;
const GAP_SAMPLES: u32 = (SAMPLE_RATE as f32 * 0.003) as u32;
const DUTY_CYCLE: f32 = 0.25;

const SFX_AMPLITUDE: f32 = 0.35;

include!("bgm_score.rs");

const TOTAL_BEATS: f32 = 512.0;

struct SampleNote {
    start: u64,
    end: u64,
    freq: f32,
    duty: f32,
    amplitude: f32,
}

struct PolySource {
    sample_idx: u64,
    total_samples: u64,
    notes: Vec<SampleNote>,
    active_start: usize,
}

impl PolySource {
    fn new() -> Self {
        let samples_per_beat = BEAT_DURATION * SAMPLE_RATE as f32;
        let total_samples = (TOTAL_BEATS * samples_per_beat) as u64;

        let notes: Vec<SampleNote> = NOTES
            .iter()
            .map(|&(start_beat, dur_beats, freq)| {
                let start = (start_beat * samples_per_beat) as u64;
                let end = ((start_beat + dur_beats) * samples_per_beat) as u64;
                let (duty, amplitude) = if freq < 200.0 {
                    (0.5, 0.10)
                } else if freq < 600.0 {
                    (0.25, 0.08)
                } else {
                    (0.25, 0.12)
                };
                SampleNote { start, end, freq, duty, amplitude }
            })
            .collect();

        Self {
            sample_idx: 0,
            total_samples,
            notes,
            active_start: 0,
        }
    }
}

impl Iterator for PolySource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let pos = self.sample_idx % self.total_samples;

        if pos == 0 && self.sample_idx > 0 {
            self.active_start = 0;
        }

        while self.active_start < self.notes.len()
            && self.notes[self.active_start].end <= pos
        {
            self.active_start += 1;
        }

        let mut value: f32 = 0.0;

        for note in &self.notes[self.active_start..] {
            if note.start > pos {
                break;
            }
            if note.end <= pos {
                continue;
            }
            let note_elapsed = pos - note.start;
            let note_len = note.end - note.start;
            if note_elapsed >= note_len.saturating_sub(GAP_SAMPLES as u64) {
                continue;
            }
            let period = SAMPLE_RATE as f32 / note.freq;
            let phase = (self.sample_idx as f32 % period) / period;
            value += if phase < note.duty { note.amplitude } else { -note.amplitude };
        }

        self.sample_idx += 1;
        Some(value)
    }
}

impl Source for PolySource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub enum Sfx {
    Move,
    Rotate,
    HardDrop,
    Hold,
    Lock,
    LineClear(u32),
    LevelUp,
    GameOver,
    MenuMove,
    MenuSelect,
    Pause,
    Resume,
    TSpinMini,
    TSpin,
    TSpinClear(u32),
    AllClear,
    Combo(u32),
    BackToBack,
    Clear,
    MenuBack,
    GarbageReceived,
    VersusWin,
    VersusLose,
}

impl Sfx {
    fn notes(&self) -> Vec<(f32, u32)> {
        match self {
            Sfx::Move => vec![(440.0, 20)],
            Sfx::Rotate => vec![(523.0, 25), (659.0, 25)],
            Sfx::HardDrop => vec![(220.0, 25), (110.0, 40)],
            Sfx::Hold => vec![(587.0, 30), (784.0, 30)],
            Sfx::Lock => vec![(247.0, 25), (220.0, 35)],
            Sfx::LineClear(n) => match n {
                1 => vec![(523.0, 50), (659.0, 60)],
                2 => vec![(523.0, 40), (659.0, 40), (784.0, 55)],
                3 => vec![(523.0, 35), (659.0, 35), (784.0, 35), (1047.0, 60)],
                _ => vec![(784.0, 40), (988.0, 40), (1175.0, 40), (1568.0, 100)],
            },
            Sfx::LevelUp => vec![
                (523.0, 50),
                (659.0, 50),
                (784.0, 50),
                (1047.0, 80),
                (1319.0, 100),
            ],
            Sfx::GameOver => vec![(440.0, 150), (370.0, 150), (311.0, 150), (247.0, 300)],
            Sfx::MenuMove => vec![(660.0, 15)],
            Sfx::MenuSelect => vec![(523.0, 40), (784.0, 40), (1047.0, 60)],
            Sfx::Pause => vec![(400.0, 60), (300.0, 80)],
            Sfx::Resume => vec![(300.0, 60), (400.0, 80)],
            Sfx::TSpinMini => vec![(659.0, 30), (784.0, 30), (659.0, 40)],
            Sfx::TSpin => vec![(523.0, 35), (784.0, 35), (1047.0, 50)],
            Sfx::TSpinClear(n) => match n {
                1 => vec![(659.0, 40), (784.0, 40), (1047.0, 60)],
                2 => vec![(659.0, 35), (784.0, 35), (1047.0, 35), (1319.0, 70)],
                _ => vec![(784.0, 35), (1047.0, 35), (1319.0, 35), (1568.0, 80)],
            },
            Sfx::AllClear => vec![
                (1047.0, 50),
                (1319.0, 50),
                (1568.0, 50),
                (2093.0, 50),
                (1568.0, 40),
                (2093.0, 80),
            ],
            Sfx::Combo(n) => {
                let base = 523.0 + *n as f32 * 50.0;
                vec![(base, 25), (base * 1.25, 35)]
            }
            Sfx::BackToBack => vec![(880.0, 30), (1047.0, 30), (1319.0, 50)],
            Sfx::Clear => vec![
                (784.0, 80), (988.0, 80), (1175.0, 80),
                (1568.0, 100), (1175.0, 60), (1568.0, 150),
            ],
            Sfx::MenuBack => vec![(523.0, 30), (392.0, 50)],
            Sfx::GarbageReceived => vec![(200.0, 40), (150.0, 60)],
            Sfx::VersusWin => vec![
                (784.0, 80), (988.0, 80), (1175.0, 80),
                (1568.0, 100), (1175.0, 60), (1568.0, 200),
            ],
            Sfx::VersusLose => vec![(440.0, 150), (370.0, 150), (311.0, 200), (247.0, 350)],
        }
    }
}

struct SfxSource {
    sample_rate: u32,
    sample_idx: u64,
    note_idx: usize,
    note_sample: u32,
    notes: Vec<(f32, u32)>,
    total_samples: u64,
}

impl SfxSource {
    fn new(notes: Vec<(f32, u32)>) -> Self {
        let total_samples: u64 = notes
            .iter()
            .map(|&(_, ms)| (SAMPLE_RATE as u64 * ms as u64) / 1000)
            .sum();
        Self {
            sample_rate: SAMPLE_RATE,
            sample_idx: 0,
            note_idx: 0,
            note_sample: 0,
            notes,
            total_samples,
        }
    }

    fn current_note_samples(&self) -> u32 {
        let (_, ms) = self.notes[self.note_idx];
        (SAMPLE_RATE as u64 * ms as u64 / 1000) as u32
    }
}

impl Iterator for SfxSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.note_idx >= self.notes.len() {
            return None;
        }

        let note_total = self.current_note_samples();
        let (freq, _) = self.notes[self.note_idx];

        let value = if freq <= 0.0 {
            0.0
        } else {
            let period = self.sample_rate as f32 / freq;
            let phase = (self.sample_idx as f32 % period) / period;
            if phase < DUTY_CYCLE {
                SFX_AMPLITUDE
            } else {
                -SFX_AMPLITUDE
            }
        };

        self.sample_idx += 1;
        self.note_sample += 1;

        if self.note_sample >= note_total {
            self.note_sample = 0;
            self.note_idx += 1;
        }

        Some(value)
    }
}

impl Source for SfxSource {
    fn current_frame_len(&self) -> Option<usize> {
        if self.note_idx >= self.notes.len() {
            return Some(0);
        }
        let remaining_in_note = self.current_note_samples() - self.note_sample;
        Some(remaining_in_note as usize)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let micros = self.total_samples * 1_000_000 / self.sample_rate as u64;
        Some(Duration::from_micros(micros))
    }
}

pub struct MusicPlayer {
    _stream: OutputStream,
    sink: Sink,
    sfx_sink: Sink,
    bgm_enabled: bool,
    sfx_enabled: bool,
    bgm_paused: bool,
}

impl MusicPlayer {
    pub fn new() -> Option<Self> {
        let (stream, stream_handle) = init_output_stream()?;
        let sink = Sink::try_new(&stream_handle).ok()?;
        let sfx_sink = Sink::try_new(&stream_handle).ok()?;
        sink.pause();

        Some(Self {
            _stream: stream,
            sink,
            sfx_sink,
            bgm_enabled: true,
            sfx_enabled: true,
            bgm_paused: false,
        })
    }

    pub fn bgm_enabled(&self) -> bool {
        self.bgm_enabled
    }

    pub fn sfx_enabled(&self) -> bool {
        self.sfx_enabled
    }

    pub fn toggle_bgm(&mut self) {
        self.bgm_enabled = !self.bgm_enabled;
        if self.bgm_enabled && !self.bgm_paused {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn toggle_sfx(&mut self) {
        self.sfx_enabled = !self.sfx_enabled;
    }

    pub fn play(&mut self) {
        self.bgm_paused = false;
        self.sink.clear();
        self.sink.append(PolySource::new());
        if self.bgm_enabled {
            self.sink.play();
        }
    }

    pub fn pause(&mut self) {
        self.bgm_paused = true;
        self.sink.pause();
    }

    pub fn resume(&mut self) {
        self.bgm_paused = false;
        if self.bgm_enabled {
            self.sink.play();
        }
    }

    pub fn stop(&mut self) {
        self.bgm_paused = false;
        self.sink.pause();
        self.sink.clear();
    }

    pub fn play_sfx(&self, sfx: Sfx) {
        if !self.sfx_enabled {
            return;
        }
        self.sfx_sink.clear();
        self.sfx_sink.append(SfxSource::new(sfx.notes()));
        self.sfx_sink.play();
    }
}
