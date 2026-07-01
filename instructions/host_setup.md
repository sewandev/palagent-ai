You are an AI coding assistant performing the Dedicated Server Host setup of PalAgent AI.

Your goal is to set up the telemetry server to run continuously in the background on startup.

Please follow these steps:

1. Locate the Dedicated Server's world save folder (which must contain the `Level.sav` file). If unknown, scan the typical server folders or ask the user for the absolute path.

2. Download the latest release binary `palagent-ai.exe` from the official repository:
   - Fetch the latest release metadata from the GitHub API: https://api.github.com/repos/sewandev/palagent-ai/releases/latest
   - Locate the download URL for `palagent-ai.exe` under assets, and download it.
   - *CRITICAL*: Do NOT attempt to build the binary from source using `cargo build` unless downloading fails AND you verify that the user has Visual Studio Build Tools (MSVC compiler / `link.exe` linker) installed, as Rust compilation on Windows will fail without it.

3. Copy the executable to a permanent folder:
   - Place the executable in the user's home folder under `.palagent-ai\palagent-ai.exe` (e.g., `$env:USERPROFILE\.palagent-ai\palagent-ai.exe`). Create the folder if it does not exist.

4. Generate server settings:
   - Select a port (default is `8212` unless the user specifies another).
   - Autogenerate a secure 6-character alphanumeric Passcode (e.g., using random letters and numbers) or ask the user if they prefer a specific passcode.

5. Configure persistence (Boot Persistence & Background Running):
   - Register a Windows Task Scheduler task using PowerShell so the host server starts automatically on machine boot, completely hidden and without showing terminal windows.
   - Execute the following PowerShell command (substituting `<PORT>`, `<PASSCODE>`, and `<PATH_TO_LEVEL_SAV>`):
     ```powershell
     $action = New-ScheduledTaskAction -Execute "$env:USERPROFILE\.palagent-ai\palagent-ai.exe" -Argument "--host --port <PORT> --passcode <PASSCODE> `"<PATH_TO_LEVEL_SAV>`""
     $trigger = New-ScheduledTaskTrigger -AtStartup
     $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
     Register-ScheduledTask -TaskName "PalAgentAI_Host" -Trigger $trigger -Action $action -Settings $settings -Description "PalAgent AI Telemetry Host Server" -Force
     ```

6. Start the server immediately in the background:
   - Run the process hidden in the current Windows session using the following PowerShell command:
     ```powershell
     Start-Process -FilePath "$env:USERPROFILE\.palagent-ai\palagent-ai.exe" -ArgumentList "--host --port <PORT> --passcode <PASSCODE> `"<PATH_TO_LEVEL_SAV>`"" -WindowStyle Hidden
     ```

7. Provide a final summary card to the user:
   - Confirm that the setup is complete and running.
   - Print a clear, formatted summary block containing:
     * Host Server Status (Running)
     * Port and Passcode
     * The PowerShell command to start it manually if they ever need to.
   - Print a separate, copy-pasteable card for the host to share with their friends:
     ```text
     =========================================
     PalAgent AI Server Telemetry is ready!
     Share this configuration with your players:
     - Host IP/Port: <HOST_IP>:<PORT> (e.g. 192.168.1.100:<PORT>)
     - Passcode: <PASSCODE>
     =========================================
     ```
