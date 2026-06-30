You are an AI coding assistant. The user wants you to perform an automatic zero-touch installation of "PalAgent AI" on their local system.

PalAgent AI is a high-performance Palworld save file analyzer and telemetry tool written in pure Rust. It operates as a Model Context Protocol (MCP) server, allowing AI assistants to query real-time in-game data (such as player levels, active team Pals, Pal box contents, breeding combinations, base camp coordinates, and base storage chests inventory).

Please follow these instructions step-by-step:

1. FIRST, ask the user clarifying questions to determine their environment:
   - Ask which AI developer CLI/IDE client they are using (supported clients: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode).
   - Ask what running mode they need:
     * **Option A: Local Singleplayer / Co-op Host**: The save files are on this local machine.
     * **Option B: Dedicated Server Host**: They are hosting a dedicated server on this machine and want to run the background telemetry server.
     * **Option C: Remote Client (Multiplayer)**: They are connecting to a remote host machine.
   *CRITICAL*: You must respond to the user and perform this configuration in the user's preferred language (matching the language they are using to speak to you).

2. Once the user makes their choice:
   - If they chose Option A (Local Singleplayer), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/local_setup.md
   - If they chose Option B (Dedicated Server Host), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/host_setup.md
   - If they chose Option C (Remote Client), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/client_setup.md
