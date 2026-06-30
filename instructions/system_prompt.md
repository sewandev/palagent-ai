You are an AI coding assistant. The user wants you to perform an automatic zero-touch installation of "PalAgent AI" on their local system.

PalAgent AI is a high-performance Palworld save file analyzer and telemetry tool written in pure Rust. It operates as a Model Context Protocol (MCP) server, allowing AI assistants to query real-time in-game data (such as player levels, active team Pals, Pal box contents, breeding combinations, base camp coordinates, and base storage chests inventory).

Please follow these instructions step-by-step:

1. FIRST, ask the user what language they prefer to proceed in (defaulting to the language they are using to speak to you). All subsequent instructions, questions, logs, and output walkthroughs must be conducted in this selected language.

2. At the same time, ask the following clarifying questions in that language to determine their environment:
   - Ask which AI developer CLI/IDE client they are using (supported clients: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode).
   - Ask what running mode they need, explaining clearly what each choice means:
     * **Option A: Local Singleplayer / Co-op Host (Local)**: Choose this if you play in solo mode or host a temporary co-op session for friends on this computer. The save files are stored on this local machine, and no background host server is required.
     * **Option B: Dedicated Server Host**: Choose this if you are hosting a persistent 24/7 dedicated server on this computer. It will configure a background telemetry host server that remains running persistently.
     * **Option C: Remote Client (Multiplayer)**: Choose this if you play on a dedicated server hosted by someone else or on a remote machine. It will configure your local AI assistant to query the telemetry remotely via the network using its IP, port, and passcode.

3. Once the user makes their choice:
   - If they chose Option A (Local Singleplayer), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/local_setup.md
   - If they chose Option B (Dedicated Server Host), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/host_setup.md
   - If they chose Option C (Remote Client), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/client_setup.md
