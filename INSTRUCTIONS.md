You are an AI coding assistant. The user wants you to perform an automatic zero-touch installation of "PalAgent AI" on their local system.

PalAgent AI is a high-performance Palworld save file analyzer and telemetry tool written in pure Rust. It operates as a Model Context Protocol (MCP) server, allowing AI assistants to query real-time in-game data (such as player levels, active team Pals, Pal box contents, breeding combinations, base camp coordinates, and base storage chests inventory).

Please follow these instructions step-by-step:

1. FIRST, ask the user the following clarifying questions in their preferred language (defaulting to the language they are using to speak to you):
   - Ask which AI developer CLI/IDE client they are using (supported clients: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode).
   - Ask what running mode they need:
     * **Singleplayer / Co-op Host (Local)**: The save files are on this local machine.
     * **Dedicated Server / Multiplayer Client (Remote)**: They connect to a remote host machine. If they choose this, ask for:
       - The Host's IP and port (e.g. `192.168.1.100:8212`).
       - The security Passcode configured on the host.
       - The player's private Player UID (e.g. `00000000-0000-0000-0000-000000000001`).
       - *Note*: Remind them that the remote host must have `palagent-ai` running in `--host` mode for this connection to work.
       - *How to get the Player UID*: If the user does not know their Player UID, you can find it automatically by first running a client query to the host without a UID:
         `palagent-ai.exe --connect <HOST_IP_PORT> --passcode <PASSCODE>`
         This will return the full player list. Find the user's nickname in the list and extract their Player UID. Alternatively, they can ask the server administrator.

2. Once you have the answers, download the latest release binary `palagent-ai.exe` from the official repository:
   - Check the GitHub API at: https://api.github.com/repos/sewandev/palagent-ai/releases/latest
   - Parse the assets, find the `palagent-ai.exe` download URL, and download it to a temporary directory in the workspace or run tools directly.

3. Run the setup command for the user's selected client:
   - Command: `palagent-ai.exe setup <agent-slug>`
   - For example, if they use Cursor: `palagent-ai.exe setup cursor`
   - Wait for the setup to complete and inspect the console output.

4. Configure the MCP arguments:
   - For **Local Mode**, setup configures this automatically.
   - For **Remote Client Mode**, modify the registered MCP configuration file. You must add the arguments to connect to the remote host. For example:
     `"args": ["mcp", "--connect", "<HOST_IP_PORT>", "--passcode", "<PASSCODE>", "--player-uid", "<PLAYER_UID>"]`
     Verify that the configuration file (e.g. `mcp.json` or `mcp_config.json` depending on their client) is correctly updated.

5. Verify that the MCP server configurations and skills are correctly installed.
   - Check that the skills directory `~/.gemini/config/skills/palagent-ai` (for Antigravity) was created and that `SKILL.md` is present.

6. Finally, instruct the user to reload or restart their IDE/CLI so that the new MCP server is loaded. At the same time, suggest concrete test queries for them to try as soon as they reload (such as asking for their character nickname, their active world name, the first 5 slots of their backpack, or their highest-level Pal) so they can verify the installation is working correctly.
