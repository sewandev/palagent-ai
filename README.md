<p align="center">
  <h1 align="center">PalSync AI LiveAgent</h1>
</p>

<p align="center">Real-time telemetry, monitoring, breeding and inventory search CLI for Palworld.</p>

<p align="center">
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml"><img alt="Build" src="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/codeql.yml"><img alt="CodeQL" src="https://github.com/sewandev/palagent-ai/actions/workflows/codeql.yml/badge.svg" /></a>
  <img alt="Version" src="https://img.shields.io/github/v/release/sewandev/palagent-ai?style=flat-square&label=version&color=blueviolet" />
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

> [!TIP]
> **Easy installation using the AI itself (the AI will do everything for you)**
>
> *1. Download the latest release from `https://api.github.com/repos/sewandev/palagent-ai/releases/latest`.*
> *2. Run the setup command for `<name_cli>`.*
> *3. Verify that the palagent-ai MCP server is ready.*
> *4. Tell the user to restart or reload their CLI/IDE.*
> *5. Inform the user they can now ask about their Palworld stats directly.*

## Installation & Setup

```bash
# Build from source (requires Rust)
git clone https://github.com/sewandev/palagent-ai.git
cd palagent-ai
cargo build --release

# Run the analyzer report
./target/release/palagent-ai.exe

# Register as an MCP Server and install skills automatically in Antigravity CLI
./target/release/palagent-ai.exe setup antigravity-cli
```

> [!IMPORTANT]
> **Palworld Save Decompression**: This tool requires the Oodle decompression library (`oo2core_9_win64.dll`) to decompress Palworld's GVAS save files. The tool automatically searches for it in your Palworld game directory. If not found, copy it next to the compiled executable.

---

## Features

- **Multi-world Auto-detection & Interactive Menu** — Automatically scans your LocalAppData to find the most recently updated Palworld save file. If multiple worlds exist, run with `--select-world` to choose interactively.
- **Game Mode Resiliency** — Auto-detects whether the save is a **Singleplayer** world, a **Co-op Multiplayer** world, or a **Dedicated Server**.
- **Real-time Server/Client Sinking** — Connect clients remotely to query their own stats from a Host machine without requiring direct file access.
- **Client Privacy Isolation** — In multiplayer/host modes, players can isolate reports using their `--player-uid`, protecting other players' inventories and Pals from unauthorized views.
- **Palbox & Base Camps Parsing** — Full extraction of Palbox offline stored Pals, Base Camp locations (coordinates, level, owner guild), and Guild memberships (members, leader).
- **Subsecond Performance** — Written in pure Rust. Utilizes byte-signature scanning directly in memory rather than parsing a massive JSON AST, making queries execute in under 1 second.
- **LLM-Friendly JSON Outputs** — Every command can output pretty-printed JSON when run with `--json`, making it perfect for feeding live data into AI coding assistants.

---

## Running Modes

### 1. Singleplayer Mode (Local)
No configuration or server setup required. Directly accesses your local save games.
```bash
# General report
palagent-ai.exe

# Search for "Berries" in all base chests
palagent-ai.exe --search-chest "Berries"

# Monitor Pal status (SAN, Hunger, HP)
palagent-ai.exe --monitor
```

### 2. Host Mode (Sync Server)
Spins up a lightweight HTTP server on port `8212` to stream telemetry securely to clients.
```bash
# Start the host server with a security passcode
palagent-ai.exe --host --port 8212 --passcode MYPASSCODE
```

### 3. Client Mode (Remote Telemetry)
Queries data remotely from the host server. Automatically filters all results by your Player UID.
```bash
# Connect and get your private player report
palagent-ai.exe --connect 192.168.1.100:8212 --passcode MYPASSCODE --player-uid <YOUR_UID>

# Get JSON formatted active quest progress remotely
palagent-ai.exe --connect 192.168.1.100:8212 --passcode MYPASSCODE --player-uid <YOUR_UID> --progress --json
```

---

## Command Reference

| Flag | Subcommand | Description |
| --- | --- | --- |
| `-t`, `--time` | In-game Time | Current in-game day, clock time, and day/night status. |
| `-s`, `--settings` | World Settings | Current server configuration and game difficulty metrics. |
| `-c`, `--search-chest` | Chest Finder | Locate specific items across all storage containers in the base. |
| `-b`, `--breeding` | Breeding Helper | Lists available males/females and lists their breeding combinations. |
| `-p`, `--progress` | Progress Metrics | Total notes read, fast travels unlocked, and Pal bonus capture tracker. |
| `-m`, `--monitor` | Base Monitor | Real-time health, satiety, sanity, and status of your active Pals. |
| `-a`, `--analyzer` | IV Analyzer | Deep dive into your Pals' levels, genders, passive skills, and IV stats (HP, Attack, Defense). |
| `--list-worlds` | Save Directory | Lists all detected local save files with their modification dates. |
