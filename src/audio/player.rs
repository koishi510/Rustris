use rodio::{OutputStream, Sink};

use super::Sfx;
use super::synth::{PolySource, SfxSource};

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
