# Rustris

[![Build](https://img.shields.io/github/actions/workflow/status/koishi510/Rustris/rust.yml?branch=main&style=flat-square)](https://github.com/koishi510/Rustris/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/rustris?style=flat-square)](https://crates.io/crates/rustris)
[![Release](https://img.shields.io/github/v/release/koishi510/Rustris?style=flat-square)](https://github.com/koishi510/Rustris/releases)
[![License](https://img.shields.io/badge/license-GPL--3.0-green?style=flat-square)](./LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

A guideline-compliant terminal Tetris written in Rust.

## Install

```sh
cargo install rustris
```

Or build from source:

```sh
git clone https://github.com/koishi510/Rustris.git
cd Rustris
cargo run --release
```

> Requires Rust 2024 edition (1.85+). On Linux, `libasound2-dev` (or equivalent) is needed for audio support.

## Game Modes

| Mode     | Objective                                                    |
| -------- | ------------------------------------------------------------ |
| Marathon | Clear a target number of lines (default 150)                 |
| Sprint   | Clear lines (default 40) as fast as possible                 |
| Ultra    | Score as high as possible within a time limit (default 120s) |
| Endless  | Play with no goal until game over                            |

## Controls

| Key          | Action                   |
| ------------ | ------------------------ |
| Left / Right | Move piece               |
| Down         | Soft drop (+1 per cell)  |
| Space        | Hard drop (+2 per cell)  |
| Up / X       | Rotate clockwise         |
| Z            | Rotate counter-clockwise |
| C            | Hold piece               |
| Esc / P      | Pause                    |

## Features

- **Super Rotation System (SRS)** with full wall kick tables
- **7-bag randomizer** (or pure random)
- **Hold piece** (once per drop)
- **Next queue** preview (1-6 pieces, configurable)
- **Ghost piece** (toggleable)
- **Lock delay** (0.5s) with move/rotate reset
- **DAS/ARR** input handling
- **Line clear animation** (toggleable)
- **Guideline scoring** - T-Spin (Mini/Full), Back-to-Back, Combo, All Clear
- **Guideline gravity** with level cap setting
- **BGM & SFX** with polyphonic playback
- **Leaderboard** - top 10 per mode, recorded only under default settings

## Settings

Each mode has its own configurable parameters:

| Setting   | Modes             | Range             |
| --------- | ----------------- | ----------------- |
| Level     | Marathon, Endless | 1-20              |
| Goal      | Marathon          | 10-300 (step 10)  |
| Goal      | Sprint            | 10-100 (step 10)  |
| Time      | Ultra             | 30-300s (step 10) |
| Level Cap | Marathon, Endless | 1-20 or None      |

Shared settings: Next count, Ghost, Line clear animation, Bag randomizer, BGM, SFX.

## Project Structure

```
src/
  main.rs          Entry point, game loop, menu and input handling
  game.rs          Game state, scoring, line clears, lock delay, gravity
  piece.rs         Piece/Bag structs, SRS data (rotation states, kick tables)
  render.rs        Terminal rendering (board, panels, menus, overlays)
  records.rs       Leaderboard persistence (JSON via serde)
  audio.rs         BGM and SFX playback
  settings.rs      Settings struct and defaults
  tetris_notes.rs  BGM note/melody data
```

## Dependencies

- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation
- [rand](https://crates.io/crates/rand) - Bag shuffling and random generation
- [rodio](https://crates.io/crates/rodio) - Audio playback
- [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) - Record serialization
- [dirs](https://crates.io/crates/dirs) - Platform data directory resolution

## License

[GPL-3.0](./LICENSE)
