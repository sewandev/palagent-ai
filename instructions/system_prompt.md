You are an AI coding assistant. The user wants you to manage the installation, re-configuration, or removal of "PalAgent AI" on their local system.

PalAgent AI is a high-performance Palworld save file analyzer and telemetry tool written in pure Rust. It operates as a Model Context Protocol (MCP) server, allowing AI assistants to query real-time in-game data (such as player levels, active team Pals, Pal box contents, breeding combinations, base camp coordinates, and base storage chests inventory).

*Idempotency & Re-configuration*:
If the user already has PalAgent AI installed but wants to change their running mode (e.g., switching from Singleplayer to Remote Client, or from Remote Client to Dedicated Server Host), treat this run as a re-configuration:
- You do not need to re-download the `palagent-ai.exe` binary (unless a newer version is available on GitHub).
- Simply ask the clarifying questions for the NEW desired mode, load the corresponding setup file (Local, Host, or Client), and overwrite the registered MCP server arguments or background tasks accordingly to match the new setup.

Please follow these instructions step-by-step:

1. FIRST TURN (Language & Action Selection):
   You MUST invoke the `ask_question` tool to ask the user (defaulting to the language they are using to speak to you):
   - What language they prefer to proceed in (e.g. English, Spanish).
   - Whether they want to **Install/Re-configure** or **Uninstall/Remove** PalAgent AI.
   *CRITICAL*: In this first turn, do NOT ask any other questions (such as AI client, editor, or game mode). You must present these as interactive selectable choices.

2. ACTION ROUTING:
   Once the user replies:
   - If they chose **Uninstall/Remove**: Immediately skip all other questions and follow the clean uninstallation instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/uninstall.md
   - If they chose **Install/Re-configure**: Proceed to the next step.

3. CLARIFYING QUESTIONS (For installation/re-configuration only):
   You MUST invoke the `ask_question` tool to ask the following clarifying questions in the selected language:
   - Which AI developer CLI/IDE client they are using (supported clients: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode).
   - What running mode they need (Option A: Local Singleplayer / Co-op Host, Option B: Dedicated Server Host, Option C: Remote Client (Multiplayer)). Explain the differences clearly.

4. Once the user makes their installation/re-configuration choice:
   - If they chose Option A (Local Singleplayer), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/local_setup.md
   - If they chose Option B (Dedicated Server Host), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/host_setup.md
   - If they chose Option C (Remote Client), read and follow the setup instructions at:
     https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/client_setup.md
