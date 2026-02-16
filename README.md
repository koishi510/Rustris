# Rustris

[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-green?style=for-the-badge)](./LICENSE)

A terminal-based Tetris game written in Rust, following the [Tetris Guideline](https://tetris.wiki/Tetris_Guideline).

## Features

- **Super Rotation System (SRS)** with full wall kick tables
- **7-bag randomizer**
- **Hold piece** (C key, once per drop)
- **6-piece next queue** preview
- **Ghost piece** showing drop destination
- **Lock delay** (0.5s) with move reset (Infinity)
- **Guideline scoring**: T-Spin (Mini/Full), Back-to-Back, Combo, All Clear
- **10 speed levels** with automatic progression

## Controls

| Key          | Action                   |
| ------------ | ------------------------ |
| Left / Right | Move piece               |
| Down         | Soft drop (+1 per cell)  |
| Space        | Hard drop (+2 per cell)  |
| Up / X       | Rotate clockwise         |
| Z            | Rotate counter-clockwise |
| C            | Hold piece               |
| Esc          | Pause / Resume           |
| R            | Restart (on game over)   |
| Q            | Quit                     |

## Build & Run

Requires Rust 2024 edition (1.85+).

```sh
cargo run --release
```

## Project Structure

```
src/
  main.rs    - Entry point, terminal setup, game loop
  game.rs    - Game state, scoring, line clears, lock delay
  piece.rs   - Piece/Bag structs, SRS data (rotation states, kick tables)
  render.rs  - Terminal rendering (board, panels, game over overlay)
```

## Dependencies

- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation
- [rand](https://crates.io/crates/rand) - 7-bag shuffling
