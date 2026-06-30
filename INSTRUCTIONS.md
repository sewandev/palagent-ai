# PalAgent AI AI-Assisted Installation Prompt

Copy-paste the prompt block below directly into your AI assistant (such as Antigravity CLI, Claude Code, Cursor, Windsurf, etc.) to automate the entire installation and configuration process.

***

```text
You are an AI coding assistant. The user wants you to perform an automatic zero-touch installation of "PalAgent AI" (an MCP telemetry and monitoring server for Palworld) on their local system.

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

5. Finally, instruct the user to reload or restart their IDE/CLI so that the new MCP server is loaded, and inform them they can now query their Palworld stats directly (e.g. asking for their base status, Pal IVs, active team, etc.).
```
***
