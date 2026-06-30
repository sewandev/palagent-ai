You are an AI coding assistant performing the clean uninstallation and removal of PalAgent AI.

Your goal is to delete all traces, configurations, background tasks, and files created by PalAgent AI on the user's local system.

Please follow these steps:

1. Terminate running processes:
   - Check if `palagent-ai.exe` is running in the background.
   - Run a command to force stop it:
     ```powershell
     Stop-Process -Name "palagent-ai" -Force -ErrorAction SilentlyContinue
     ```

2. Unregister boot persistence tasks:
   - If the user configured Dedicated Server Host mode, remove the registered task from the Windows Task Scheduler.
   - Run the following PowerShell command:
     ```powershell
     Unregister-ScheduledTask -TaskName "PalAgentAI_Host" -Confirm:$false -ErrorAction SilentlyContinue
     ```

3. Remove MCP server registrations:
   - Locate the configuration files for the user's selected AI client:
     * **Antigravity CLI**: `~/.gemini/config/mcp_config.json`
     * **Cursor**: `%APPDATA%\Cursor\User\globalStorage\mermaid-mcp\config.json` (or standard Cursor MCP storage)
     * **VS Code Copilot / Windsurf**: Respective client configuration files.
   - Read the configuration file, remove the `palagent-ai` server entry from the `mcpServers` object, and write the updated JSON back to the file.

4. Delete application folders and files:
   - Remove the permanent executable and logs directory:
     ```powershell
     Remove-Item -Path "$env:USERPROFILE\.palagent-ai" -Recurse -Force -ErrorAction SilentlyContinue
     ```
   - If using Antigravity, remove the installed skills directory:
     ```powershell
     Remove-Item -Path "$env:USERPROFILE\.gemini\config\skills\palagent-ai" -Recurse -Force -ErrorAction SilentlyContinue
     ```

5. Confirm cleanup to the user:
   - Tell the user: *"Uninstallation completed successfully! All files, folders, background processes, and AI configuration entries for PalAgent AI have been completely removed from your system. Please reload or restart your IDE/CLI to apply the changes."*
