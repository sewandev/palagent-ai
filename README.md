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
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

## Prerequisites

To use PalAgent AI, make sure you meet the following requirements:

1. **AI Assistant or CLI Platform**: You must have an AI-driven development environment installed. Examples include:
   - Antigravity CLI (`agy`)
   - Claude Code
   - OpenCode
   - VS Code Copilot
   - Cursor
   - Windsurf
   - Codex, Qwen, Kiro, etc.
   *Note: Active subscriptions are not required! You can use models that grant free quota or tokens. If you want to try this completely for free without spending anything, we recommend using **OpenCode with the ZEN model**, which is free.*

2. **Operating System**: **Windows** (64-bit).
   *Note: While Rust is multi-platform (Multi-OS), PalAgent AI is currently only tested on Windows. Contributions from the community to test and support other platforms are highly welcome!*

3. **Game & Platform**:
   - **Palworld** must be installed and updated.
   - Currently, only the **Steam** version of the game is supported.

---

## 1-Click AI Installation

To automatically install and configure PalAgent AI, simply copy and paste the following prompt directly into your favorite AI assistant or CLI chat:

```text
follow these instructions https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/system_prompt.md
```

### What this will do (at a glance):
1. **Interactive Language Setup**: The AI assistant will ask which language you prefer to communicate in.
2. **Clarifying Questions**: The assistant will ask about your editor/IDE client and your running mode:
   - **Local Singleplayer / Co-op Host**: If you play alone or host a temporary co-op session on your computer.
   - **Dedicated Server Host**: If you are hosting a 24/7 dedicated server on your computer and want telemetry running persistently in the background.
   - **Remote Client (Multiplayer)**: If you play on a server hosted by a friend or remote machine.
3. **Automated Setup**:
   - Downloads the latest release binary `palagent-ai.exe` automatically.
   - Copies it to a permanent location and registers the MCP configurations for your IDE/CLI.
   - Detects your player UID automatically (using `local-uid` to calculate it securely from your Steam session cache).
   - If hosting a dedicated server, registers a background task via Windows Task Scheduler for boot persistence.
   - Connects and validates the server settings, discovering your in-game nickname to welcome you.
