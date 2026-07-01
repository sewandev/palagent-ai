<p align="center">
  <h1 align="center">PalAgent AI</h1>
</p>

<p align="center">Real-time telemetry, monitoring, breeding and inventory search CLI & MCP Server for Palworld.</p>

<p align="center">
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml"><img alt="Build" src="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml/badge.svg" /></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

## 🚀 Overview

**PalAgent AI** connects your local AI assistant (Antigravity, Cursor, VS Code Copilot, Windsurf, Claude Code, Gemini CLI, etc.) directly to your Palworld save files. 

By exposing real-time world, base, inventory, and Palbox data via the **Model Context Protocol (MCP)**, your AI assistant can analyze statistics, recommend optimal breeding paths, calculate capture probabilities, locate items in chests, and help manage your base camps natively in your chat window.

---

## 🛠️ Installation Options

Choose the installation method that fits your preferences:

| Method | Setup Difficulty | Command | Details |
| :--- | :--- | :--- | :--- |
| **1. Chat AI Agent** | 🟢 **Super Easy** | Paste this prompt into your AI chat:<br>`follow these instructions https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/system_prompt.md` | Your AI coding assistant will clone, compile, and configure the entire database and MCP setup automatically for you. |
| **2. Interactive PowerShell** | 🟡 **Quick & Automated** | Run in PowerShell:<br>`Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12; iex (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/sewandev/palagent-ai/main/install.ps1')` | Automatically compiles the binary in release, links Oodle DLLs, and displays an interactive menu to set up your editors. |
| **3. Manual Rust Build** | 🔴 **Advanced** | `git clone https://github.com/sewandev/palagent-ai`<br>`cargo build --release`<br>`target/release/palagent-ai setup <editor>` | Build the Rust binary manually, resolve `oo2core_9_win64.dll`, populate SQLite database, and configure your MCP client manually. |

---

## 🧬 AI Capabilities & Prompts

Once the MCP server is configured in your editor, your AI assistant can retrieve live data to answer queries like:

> [!NOTE]
> * **Optimal Workloads**: *"Which of my Palbox Pals are the most efficient to assign to Mining at base camp?"* (AI reads work suitabilities, passive traits like Artisan/Serious, and recommends the best setup).
> * **Target Breeding Combos**: *"How can I breed an Anubis? Check if I have the required parents in my Palbox."* (AI calculates breeding paths, checks SQLite exceptions, and matches them against your live save file).
> * **Capture Probability**: *"What is my capture rate for a level 33 Chillet if I use Megaspheres with my current Lifmunk Statue bonus?"*
> * **Item Tracking**: *"Where did I store my Paldium Fragments? Do I have enough wood to craft a Mega Sphere?"*
> * **Palbox Monitoring**: *"Are any working Pals currently hungry or depressed?"*

---

## 📦 Database & Tools Reference

The native SQLite database is generated dynamically on first run and exposes the following Model Context Protocol (MCP) tools:

*   **`list_worlds`**: Lists all detected local singleplayer/dedicated save paths.
*   **`query_time`**: Day and night cycles of your active game.
*   **`query_settings`**: Global difficulty and damage multipliers.
*   **`search_chest`**: Search for items across all player containers.
*   **`query_breeding`**: Live breeding path matchmaking for your Palbox.
*   **`query_target_breeding`**: Find parent combinations that produce a specific child.
*   **`query_progress`**: Explorer diaries, fast travels, and Lifmunk efigies progress.
*   **`monitor_pals`**: HP, Hunger, and Sanity monitoring.
*   **`query_recipes`**: Ingredients and cost for Pal Spheres and structures.
*   **`query_active_skills`**: Cooldown, element, and damage of active moves.
*   **`query_drops`**: Drop rates of items from Pals.

---

## ⚖️ License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.
