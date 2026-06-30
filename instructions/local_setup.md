You are an AI coding assistant performing the Local Singleplayer / Co-op Host setup of PalAgent AI.

Please follow these steps:

1. Download the latest release binary `palagent-ai.exe` from the official repository:
   - Fetch the latest release metadata from the GitHub API: https://api.github.com/repos/sewandev/palagent-ai/releases/latest
   - Locate the download URL for `palagent-ai.exe` under assets, and download the executable file to a temporary directory in the workspace or run commands.

2. Run the setup command for the user's selected client:
   - Command: `palagent-ai.exe setup <agent-slug>` (e.g. `palagent-ai.exe setup cursor` if they use Cursor).
   - Wait for the setup to complete and verify the output.

3. Verify that the MCP server configurations and skills are correctly installed.
   - For example, if they use Antigravity, verify that the directory `~/.gemini/config/skills/palagent-ai/` was created and contains `SKILL.md`.

4. Instruct the user to reload or restart their IDE/CLI to load the new MCP server. At the same time, suggest concrete test queries (such as asking for their character nickname, their active world name, their inventory items, or their highest-level Pal) for them to run as soon as they reload to verify it works.
