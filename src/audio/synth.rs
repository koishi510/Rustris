use rodio::source::Source;
use std::time::Duration;

use super::{bgm::build_notes, SAMPLE_RATE, BEAT_DURATION, TOTAL_BEATS, GAP_SAMPLES, DUTY_CYCLE, SFX_AMPLITUDE};

pub(super) struct SampleNote {
    pub start: u64,
    pub end: u64,
    pub freq: f32,
    pub duty: f32,
    pub amplitude: f32,
}

pub(super) struct PolySource {
    sample_idx: u64,
    total_samples: u64,
    notes: Vec<SampleNote>,
    active_start: usize,
}

impl PolySource {
    pub fn new() -> Self {
        let samples_per_beat = BEAT_DURATION * SAMPLE_RATE as f32;
        let total_samples = (TOTAL_BEATS * samples_per_beat) as u64;

        let raw_notes = build_notes();
        let notes: Vec<SampleNote> = raw_notes
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

pub(super) struct SfxSource {
    sample_rate: u32,
    sample_idx: u64,
    note_idx: usize,
    note_sample: u32,
    notes: Vec<(f32, u32)>,
    total_samples: u64,
}

impl SfxSource {
    pub fn new(notes: Vec<(f32, u32)>) -> Self {
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
