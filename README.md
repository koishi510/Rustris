# Rustris

[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-green?style=for-the-badge)](./LICENSE)

A terminal-based Tetris game written in Rust, following the [Tetris Guideline](https://tetris.wiki/Tetris_Guideline).

## Install

```sh
cargo install rustris
```

## Features

- **Super Rotation System (SRS)** with full wall kick tables
- **7-bag randomizer** (or pure random)
- **Hold piece** (once per drop)
- **Next queue** preview (1–6 pieces, configurable)
- **Ghost piece** (toggleable)
- **Lock delay** (0.5s) with move/rotate reset
- **DAS/ARR** input handling
- **Line clear animation** (toggleable)
- **Guideline scoring**: T-Spin (Mini/Full), Back-to-Back, Combo, All Clear
- **Guideline gravity** with level cap setting
- **3 game modes**: Marathon, Sprint, Ultra
- **BGM & SFX** with polyphonic playback
- **Settings page** with configurable level, goals, timers, and more

## Game Modes

| Mode     | Objective                                            |
| -------- | ---------------------------------------------------- |
| Marathon | Clear a target number of lines (or None for endless) |
| Sprint   | Clear 40 lines as fast as possible                   |
| Ultra    | Score as high as possible within a time limit        |

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

## Build & Run

Requires Rust 2024 edition (1.85+).

```sh
cargo run --release
```

## Project Structure

```
src/
  main.rs          - Entry point, game loop, menu/settings input handling
  game.rs          - Game state, scoring, line clears, lock delay, gravity
  piece.rs         - Piece/Bag structs, SRS data (rotation states, kick tables)
  render.rs        - Terminal rendering (board, panels, menus, overlays)
  audio.rs         - BGM and SFX playback
  settings.rs      - Settings struct and defaults
  tetris_notes.rs  - BGM note/melody data
```

## Dependencies

- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation
- [rand](https://crates.io/crates/rand) - Bag shuffling and random generation
- [rodio](https://crates.io/crates/rodio) - Audio playback
