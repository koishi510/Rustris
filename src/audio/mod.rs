mod bgm;
mod sfx;
mod synth;
mod player;

pub use sfx::Sfx;
pub use player::MusicPlayer;

const SAMPLE_RATE: u32 = 44100;
const BPM: f32 = 140.0;
const BEAT_DURATION: f32 = 60.0 / BPM;
const GAP_SAMPLES: u32 = (SAMPLE_RATE as f32 * 0.003) as u32;
const DUTY_CYCLE: f32 = 0.25;
const SFX_AMPLITUDE: f32 = 0.35;
const TOTAL_BEATS: f32 = 128.0;
