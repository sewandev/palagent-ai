You are an AI coding assistant performing the Remote Client setup of PalAgent AI.

Your goal is to configure the local MCP server to connect to a remote host.

Please follow these steps:

1. Request the remote Host IP/Port (e.g. `192.168.1.100:8212`) and security Passcode from the user. Remind them that the host server must have `palagent-ai` running in `--host` mode.

2. Download the latest release binary `palagent-ai.exe` from the official repository:
   - Fetch the latest release metadata from the GitHub API: https://api.github.com/repos/sewandev/palagent-ai/releases/latest
   - Locate the download URL for `palagent-ai.exe` under assets, and download the executable file.
   - *CRITICAL*: Do NOT attempt to build the binary from source using `cargo build` unless downloading fails AND you verify that the user has Visual Studio Build Tools (MSVC compiler / `link.exe` linker) installed, as Rust compilation on Windows will fail without it.

3. Run the setup command for the user's selected client:
   - Command: `palagent-ai.exe setup <agent-slug>` (e.g. `palagent-ai.exe setup cursor` if they use Cursor).
   - Wait for the setup to complete and inspect the console output.

4. Retrieve the local Player's UID automatically:
   - Run the command: `palagent-ai.exe local-uid --json`
   - This command reads the local SteamID save cache from the user's system and returns the mathematically computed Player UID for their active session.
   - Parse the JSON output and extract the `player_uid` value.

5. Configure the MCP arguments:
   - Modify the registered MCP configuration file to add connection arguments targeting the remote host. For example:
     `"args": ["mcp", "--connect", "<HOST_IP_PORT>", "--passcode", "<PASSCODE>", "--player-uid", "<PLAYER_UID>"]`
     Use the `<PLAYER_UID>` retrieved in the previous step. Verify that the configuration file (e.g. `mcp.json` or `mcp_config.json` depending on their client) is correctly updated.

6. Validate connection & discover Character Nickname:
   - Run a test client query to check connection to the host using the user's local UID:
     `palagent-ai.exe --connect <HOST_IP_PORT> --passcode <PASSCODE> --player-uid <PLAYER_UID>`
   - If the connection succeeds and returns character stats, parse the output to find their character nickname.
   - If it fails, report the error to the user and double-check credentials.

7. Final Greeting:
   - Confirm that setup is complete.
   - Greet the user with a message like: *"Setup completed! Your character on the server is **<nickname>** and your Player UID is **<player_uid>**."*
   - Instruct them to reload or restart their IDE/CLI so the MCP server loads the remote configuration.
