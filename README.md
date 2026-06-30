<p align="center">
  <h1 align="center">PalAgent AI</h1>
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
  <img alt="Palworld Compatibility" src="https://img.shields.io/badge/Palworld%20Compatibility-100%25-green?style=flat-square" />
  <img alt="PalAgent v1.0" src="https://img.shields.io/badge/v1.0%20Release-Coming%20July%2010%2C%202026-blueviolet?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

## Overview

**PalAgent AI** is an intelligent assistant and Model Context Protocol (MCP) server for Palworld. Instead of manually inspecting your Palbox, tracking base coordinates, searching through chests, or manually entering stats into external breeding calculators and database websites, PalAgent AI connects your local AI assistant directly to your save file. 

Because the AI has real-time, comprehensive context of your active world, progress, inventory, and Pals, you can ask for tailored advice, optimal breeding combinations, workload optimization, or item locations. Every recommendation is directly customized to your actual in-game state, making gameplay management seamless and highly efficient.

---

## AI Capabilities (What you can ask)

Once configured, you can query your AI assistant naturally about your game state. Here are some examples of what you can ask:

### Deep AI Diagnostics (Examples)
* **Optimal Worker Placement**: *"Based on my current Palbox inventory, which Pals are the most efficient to assign to Mining (e.g., iron ore) and why?"* (The AI will inspect work suitabilities, passive traits like Artisan/Serious, and recommend the best setup).
* **Combat Team Analysis**: *"Which is my strongest Fire-type Pal in my Palbox, and can you explain in detail why?"* (The AI will read hidden Talent IV stats (HP/Atk/Def bonuses), combat moves, passive skills, and cross-reference them with the current meta).
* **Base Optimization & Diagnostics**: *"Which of my base workers have negative passives (like Slacker or Destructive) that are hurting my base efficiency, and who should I replace them with from my box?"*

### Telemetry & General Queries
* **Locate Stored Items**: Find where resources are located without opening every chest (e.g., *"Where is my carbon stored?"* or *"Do I have enough Pal Metal Ingots in my base camps?"*).
* **Pal Sanity & Health Monitoring**: Check on your working Pals (e.g., *"Are any of the Pals in base camp hungry?"* or *"What is the current SAN level of my active combat team?"*).
* **Optimal Breeding Matchmaking**: Calculate combinations using the Pals you actually own (e.g., *"How can I breed an Anubis with my current Pals?"* or *"Who should I pair to get a Jetragon with Swift?"*).
* **Track Captures & Explorer Logs**: Keep tabs on milestones and collectibles (e.g., *"What is my capture progress for Lamball?"* or *"How many explorer notes have I collected?"*).
* **Server Rules & In-Game Time**: Query difficulty or environmental cycles (e.g., *"What are the server multipliers?"* or *"Is it currently day or night in the game?"*).

---

## Remote Host & Client Features

PalAgent AI is built to handle multiplayer and dedicated servers natively:
* **Zero-Touch Client Identification**: The tool automatically scans your local Steam session cache to calculate your unique Player UID. No typing or copy-pasting complex IDs is required.
* **Identity Verification**: When setting up client mode, the assistant validates your connection against the host and automatically retrieves your in-game character nickname.
* **Boot Persistence for Hosts**: For dedicated server administrators, the installer can register a silent, windowless background telemetry service via Windows Task Scheduler that starts automatically on boot.

---

## Prerequisites

| Requirement | Supported Specifications | Note / Details |
| :--- | :--- | :--- |
| **AI Assistant / CLI** | Antigravity CLI, Claude Code, OpenCode, VS Code Copilot, Cursor, Windsurf, Codex, Qwen, Kiro, etc. | No active subscriptions required. |
| **Operating System** | Windows (64-bit) | Tested on Windows; community help needed for other OS. |
| **Game Client** | Palworld (Steam Version Only) | Must be installed and updated. |

> [!TIP]
> **No Subscriptions Required!**
> You can use any free tier or token-grant models of your preferred AI client. If you want to run this completely for free without spending anything, we recommend using **OpenCode with the ZEN model** (which has zero cost).

> [!IMPORTANT]
> **Windows and Steam Version Only**
> Currently, the save parsing signature matching is only tested on Windows and requires the Steam version of Palworld.

---

## 1-Click AI Installation

To automatically install and configure PalAgent AI on your machine, copy and paste this command directly into your AI assistant or CLI chat:

```text
follow these instructions https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/system_prompt.md
```

---

## How It Works (High-Level Overview)

When you copy-paste the installation prompt, your AI assistant will guide you step-by-step:

### 1. Verification & Setup
* **Language Match**: The assistant will automatically greet you and operate in your preferred language.
* **Running Modes**: You will choose one of three setups:
  * **Local Singleplayer / Co-op Host (Local)**: No background server needed; reads local save files directly.
  * **Dedicated Server Host**: Installs a persistent telemetry server running in the background via Windows Task Scheduler.
  * **Remote Client (Multiplayer)**: Connects to a remote server using the host's IP/port and passcode.

### 2. Zero-Touch Configuration
* **Autodetects Player UID**: Automatically reads your active Steam session cache via `local-uid` to calculate your secure Player GUID. No manual typing or guess-work is required.
* **Autodetects Nickname**: Connects to the host server, matches your player record, retrieves your in-game character nickname, and displays a customized welcome message.
* **Boot Persistence**: For dedicated hosts, registers the server using a hidden background process starting automatically on Windows boot.
