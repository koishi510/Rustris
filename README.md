# Rustris

[![Build](https://img.shields.io/github/actions/workflow/status/koishi510/Rustris/rust.yml?branch=main&style=flat-square&logo=github&logoColor=white)](https://github.com/koishi510/Rustris/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/rustris?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/rustris)
[![Release](https://img.shields.io/github/v/release/koishi510/Rustris?style=flat-square&logo=github&logoColor=white)](https://github.com/koishi510/Rustris/releases)
[![License](https://img.shields.io/badge/License-GPL--3.0-green?style=flat-square&logo=opensourceinitiative&logoColor=white)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85-orange?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

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

- **Super Rotation System (SRS)** with full wall kick tables (toggleable)
- **7-bag randomizer** (or pure random)
- **Hold piece** (toggleable)
- **Next queue** preview (0-6 pieces, configurable)
- **Ghost piece** (toggleable)
- **Lock delay** (0-2s, configurable) with move/rotate reset (0-30 or unlimited)
- **DAS/ARR** input handling
- **Line clear animation** (toggleable)
- **Guideline scoring** - T-Spin (Mini/Full), Back-to-Back, Combo, All Clear
- **Guideline gravity** with level cap setting
- **BGM & SFX** with polyphonic playback
- **Leaderboard** - top 10 per mode, recorded only under default settings

## Settings

| Setting   | Modes             | Range              | Default | Description                          |
| --------- | ----------------- | ------------------ | ------- | ------------------------------------ |
| Level     | Marathon, Endless | 1-20               | 1       | Starting level                       |
| Goal      | Marathon          | 10-300 (step 10)   | 150     | Lines to clear                       |
| Goal      | Sprint            | 10-100 (step 10)   | 40      | Lines to clear                       |
| Time      | Ultra             | 30-300s (step 10)  | 120s    | Time limit                           |
| Cap       | Marathon, Endless | 1-20 / INF         | 15      | Maximum level                        |
| Next      | All               | 0-6                | 6       | Next queue preview count             |
| Lock      | All               | 0.0-2.0s (step 0.1)| 0.5s   | Lock delay before piece locks        |
| Reset     | All               | 0-30 / INF         | 15      | Move reset limit during lock delay   |
| Ghost     | All               | ON / OFF           | ON      | Ghost piece visibility               |
| Anim      | All               | ON / OFF           | ON      | Line clear animation                 |
| Bag       | All               | ON / OFF           | ON      | 7-bag randomizer (OFF = pure random) |
| SRS       | All               | ON / OFF           | ON      | Super Rotation System with wall kicks |
| Hold      | All               | ON / OFF           | ON      | Hold piece                           |
| BGM       | All               | ON / OFF           | ON      | Background music                     |
| SFX       | All               | ON / OFF           | ON      | Sound effects                        |

## Project Structure

```
src/
  main.rs            Entry point, terminal init/cleanup
  ui/
    menus.rs         Mode select, settings, records screens
    game_loop.rs     Game loop, DAS input, SFX dispatch
  game/
    types.rs         GameMode, LastMove, ClearAction, timing constants
    mod.rs           Game struct and all gameplay logic
  render/
    board.rs         Main board rendering
    menus.rs         Menu/overlay rendering (pause, game over, settings, etc.)
    mod.rs           Shared render utilities, title, piece preview
  piece.rs           Piece/Bag structs, SRS data (rotation states, kick tables)
  records.rs         Leaderboard persistence (JSON via serde)
  audio.rs           BGM and SFX playback
  settings.rs        Settings struct and defaults
  bgm_score.rs       BGM note/melody data
```

## Dependencies

- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation
- [rand](https://crates.io/crates/rand) - Bag shuffling and random generation
- [rodio](https://crates.io/crates/rodio) - Audio playback
- [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) - Record serialization
- [dirs](https://crates.io/crates/dirs) - Platform data directory resolution

## License

[GPL-3.0](./LICENSE)
