# Rustris

[![Build](https://img.shields.io/github/actions/workflow/status/koishi510/Rustris/rust.yml?branch=main&style=flat-square&logo=github&logoColor=white)](https://github.com/koishi510/Rustris/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/rustris?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/rustris)
[![Release](https://img.shields.io/github/v/release/koishi510/Rustris?style=flat-square&logo=github&logoColor=white)](https://github.com/koishi510/Rustris/releases)
[![License](https://img.shields.io/badge/License-GPL--3.0-green?style=flat-square&logo=opensourceinitiative&logoColor=white)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85-orange?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

A guideline-compliant terminal Tetris with LAN multiplayer support.

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

<p align="center">
  <img src="assets/demo_solo.gif" alt="Solo gameplay">
</p>

| Mode     | Objective                                                    |
| -------- | ------------------------------------------------------------ |
| Marathon | Clear a target number of lines (default 150)                 |
| Sprint   | Clear lines (default 40) as fast as possible                 |
| Ultra    | Score as high as possible within a time limit (default 120s) |
| Endless  | Play with no goal until game over                            |
| Versus   | LAN 1v1 - send garbage lines to your opponent                |

## Versus Mode (LAN Multiplayer)

<p align="center">
  <img src="assets/demo_pvp.gif" alt="Versus gameplay">
</p>

Play 1v1 over a local network. One player hosts, the other joins.

### Quick Start

Start the game and select **Versus** mode from the menu. One player selects **Host** (enter a port), the other selects **Join** (enter `<host-ip>:<port>`). The host's LAN IP is displayed on the lobby screen.

### Garbage System

Clearing lines sends garbage to your opponent:

| Clear Type         | Attack |
| ------------------ | ------ |
| Single             | 0      |
| Double             | 1      |
| Triple             | 2      |
| Tetris             | 4      |
| T-Spin Single      | 2      |
| T-Spin Double      | 4      |
| T-Spin Triple      | 6      |
| T-Spin Mini Single | 0      |
| T-Spin Mini Double | 1      |
| Back-to-Back       | +1     |
| Perfect Clear      | 10     |

Combo bonus (added on top): 0-1 combo = +0, 2-3 = +1, 4-5 = +2, 6-7 = +3, 8-10 = +4, 11+ = +5.

Pending garbage is absorbed when you clear lines (cancel before send). Uncleared garbage is applied to your board on lock. A red bar between the two boards shows the amount of pending garbage.

### Versus Rules

- Level is fixed (no level-up during a match)
- Game does not pause; Esc opens a non-blocking Forfeit menu (gravity and network continue)
- No records are saved for Versus games

## Menu Navigation

All menus use **Up/Down** to navigate, **Enter** to select, and **Left/Right** to change mode or toggle values.

```
Main Menu
├── < Mode >          ← Left/Right to switch (works on any item)
├── Start             → Start game
├── Settings          → Settings (mode-specific + audio)
├── Records           → Leaderboard (Left/Right to switch mode)
├── Help              → Controls reference
└── Quit              → Exit

Versus Menu
├── Host Game         → Port Input
│   ├── Confirm       → Host Lobby
│   ├── Back          → Versus Menu
│   └── Menu          → Main Menu
├── Join Game         → IP Input → Port Input
│   ├── Confirm       → Client Lobby
│   ├── Back          → Previous step (Port→IP, IP→Versus Menu)
│   └── Menu          → Main Menu
└── Back              → Main Menu

Host Lobby (waiting for connection)
├── Back              → Versus Menu
└── Menu              → Main Menu

Client Lobby (connection failed)
├── Retry             → Retry connection
├── Back              → Versus Menu
└── Menu              → Main Menu

Pause (single-player, Esc/P to open)
├── Resume            → Resume game (Esc also resumes)
├── Settings          → BGM/SFX toggles
├── Help              → Controls reference
├── Retry             → Restart game
└── Menu              → Main Menu

Game Over (single-player)
├── Retry             → Restart game
└── Menu              → Main Menu

Forfeit (versus, Esc to open, non-blocking)
├── BGM               ← Left/Right/Enter to toggle
├── SFX               ← Left/Right/Enter to toggle
├── Continue          → Resume (Esc also resumes)
└── Forfeit           → Lose and end match

Versus Result
├── Rematch           → Request rematch (waiting screen)
│   ├── Back          → Result screen
│   └── Menu          → Disconnect, Main Menu
└── Menu              → Disconnect, Main Menu
```

## Controls

| Key          | Action                   |
| ------------ | ------------------------ |
| Left / Right | Move piece               |
| Down         | Soft drop (+1 per cell)  |
| Space        | Hard drop (+2 per cell)  |
| Up / X       | Rotate clockwise         |
| Z            | Rotate counter-clockwise |
| C            | Hold piece               |
| Esc / P      | Pause (Forfeit in Versus)|
| Ctrl+C       | Force quit               |

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
- **LAN Versus** - P2P TCP multiplayer with protocol handshake, garbage system, dual-board rendering, rematch support

## Settings

| Setting   | Modes             | Range              | Default | Description                          |
| --------- | ----------------- | ------------------ | ------- | ------------------------------------ |
| Level     | Marathon, Endless, Versus | 1-20       | 1       | Starting level                       |
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
  main.rs              Entry point, terminal init/cleanup
  audio/
    mod.rs             Audio constants, module exports
    bgm.rs             BGM note/melody data, cycle assembly
    sfx.rs             Sfx enum, note sequences per sound effect
    synth.rs           Polyphonic synthesis (PolySource, SfxSource)
    player.rs          MusicPlayer: BGM/SFX playback via rodio
  game/
    mod.rs             Game struct definition
    board.rs           Construction, board queries, hold, ghost, timing
    movement.rs        Piece movement, rotation (SRS), gravity, drop
    scoring.rs         T-Spin detection, line clear, scoring
    animation.rs       Line clear animation, ARE, garbage rise animation
    types.rs           GameMode, LastMove, ClearAction, timing constants
    piece.rs           Piece/Bag structs, SRS data (rotation states, kick tables)
    settings.rs        Settings struct (shared by solo and versus)
    records.rs         Leaderboard persistence (JSON via serde)
    garbage.rs         Attack calculation, garbage queue, cancel logic
    tests.rs           Unit tests (board, piece, garbage, scoring)
  net/
    mod.rs             Network module exports
    protocol.rs        NetMessage enum, protocol version, BoardSnapshot, GarbageAttack
    transport.rs       Connection: frame encoding/decoding, non-blocking TCP I/O, timeout/length guard
    host.rs            LAN IP detection, TCP listener (non-blocking accept)
    client.rs          TCP connect with timeout
  render/
    mod.rs             Render module exports
    common.rs          Shared render utilities, title, piece preview
    board.rs           Single-player board rendering
    menus.rs           Menu/overlay rendering (pause, game over, settings, etc.)
    versus.rs          Dual-board rendering, lobby/countdown/result screens
  ui/
    mod.rs             UI module exports
    app.rs             Application loop, versus flow dispatch
    input.rs           Key handling, DAS/ARR, gravity, lock delay, menu helpers
    session.rs         Single-player game loop, pause, game over, records
    versus.rs          Versus game loop, lobby, handshake, countdown, garbage, rematch
    menus/
      mod.rs           Menu module exports
      modes.rs         Mode select screen, records viewer
      settings.rs      Settings menu (in-game and full)
      versus.rs        Versus Host/Join sub-menus with port/address input
```

## Dependencies

- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation
- [rand](https://crates.io/crates/rand) - Bag shuffling and random generation
- [rodio](https://crates.io/crates/rodio) - Audio playback
- [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) - Record and network message serialization
- [dirs](https://crates.io/crates/dirs) - Platform data directory resolution

## License

[GPL-3.0](./LICENSE)
