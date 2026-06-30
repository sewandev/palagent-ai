You are an AI coding assistant. The user wants you to perform an automatic zero-touch installation of "PalAgent AI" on their local system.

PalAgent AI is a high-performance Palworld save file analyzer and telemetry tool written in pure Rust. It operates as a Model Context Protocol (MCP) server, allowing AI assistants to query real-time in-game data (such as player levels, active team Pals, Pal box contents, breeding combinations, base camp coordinates, and base storage chests inventory).

Please follow these instructions step-by-step:

1. FIRST, ask the user the following clarifying questions in their preferred language (defaulting to the language they are using to speak to you):
   - Ask which AI developer CLI/IDE client they are using (supported clients: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode).
   - Ask if they currently have Palworld installed on Windows and if they have at least one active game save/world.

2. Once you have the answers, download the latest release binary `palagent-ai.exe` from the official repository:
   - Check the GitHub API at: https://api.github.com/repos/sewandev/palagent-ai/releases/latest
   - Parse the assets, find the `palagent-ai.exe` download URL, and download it to a temporary directory in the workspace or run tools directly.

3. Run the setup command for the user's selected client:
   - Command: `palagent-ai.exe setup <agent-slug>`
   - For example, if they use Cursor: `palagent-ai.exe setup cursor`
   - Wait for the setup to complete and inspect the console output.

4. Verify that the MCP server configurations and skills are correctly installed.
   - For example, if they use Antigravity, check that the skills directory `~/.gemini/config/skills/palagent-ai` was created and that `SKILL.md` is present.

5. Instruct the user to reload or restart their IDE/CLI so that the new MCP server is loaded.

6. PROPOSE A TEST QUERY: Once the restart is complete, suggest the user to run a quick test query to verify that the MCP server is working correctly. Give them concrete examples of what they can ask, such as:
   - "What is my character nickname?"
   - "What is the name of my active world/save?"
   - "What items do I have in the first 5 slots of my backpack?"
   - "Which of my Pals has the highest level?"
   Offer to run the corresponding MCP tool query immediately if they agree, and display the result.
