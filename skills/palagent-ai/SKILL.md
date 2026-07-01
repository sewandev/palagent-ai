---
name: palagent-ai
description: Extract telemetry, stats, IVs, breeding combinations, and base camps from Palworld save files.
---

# PalAgent AI Skill

This skill allows the agent to interact with the PalAgent AI MCP server, CLI, and query real-time Palworld statistics.
Use the `palagent-ai` tools when:
- The user asks for the status of base camps or Palbox.
- The user wants to analyze Pal IVs, stats, or passive skills (optionally filtering by trait, level, or gender).
- The user requests breeding combinations or wants to find a multi-generational breeding path to get a target Pal (breeding pathfinder).
- The user needs to locate items in base chests.
- The user wants to run multiple consecutive commands fast using the memory-cached `--interactive` console mode.

## Mandatory Data Enrichment & External Knowledge Policy

1. **Never Rely on Static Internal Knowledge**:
   - Palworld is frequently updated. Your internal knowledge (frozen around early 2025) is likely outdated for newer patches, Pals, items, or stats (e.g. Sakurajima update or later).
   - Never answer breeding suggestions, passive viability, base layouts, or item utilities purely from memory. Always perform live web queries to cross-reference telemetry with the latest meta.

2. **Required Web Search (Cross-Referencing)**:
   - When the MCP tool returns telemetry (like a list of Pal names, item names, stats, or guild status), enrich this raw data by performing active web searches using the `search_web` tool on resources like `palworld.gg` or the official Palworld Wiki.
   - For example, if the user asks *"Are the passive skills on my Anubis good?"*, use the MCP to read the passive skills, but immediately run a web search to check the current meta for Anubis builds on `palworld.gg` before replying.

3. **Handling Unknown Data**:
   - If the MCP server returns a Pal, item, or status ID that you do not recognize (due to post-2025 game updates), do not assume it is an error or hallucination. Instantly run a web search using the exact ID or name to identify it and explain its properties to the user.

4. **Prohibit Lazy Thinking**:
   - Do not summarize raw JSON outputs without analyzing what they mean in the current state of the game. Always provide actionable advice (e.g., advising which Pals to breed to get a desired trait, indicating if an IV is close to maximum, or suggesting how to optimize camp workloads based on current patch data).

## Version Detection & Comparison Policy

1. **Verify Local Game Version**:
   - Check if any configuration files, setup logs, or telemetry outputs contain game version or GVAS save header version details.
   - If version data is not present in the telemetry, look for typical game logs or executables on the user's system to detect the version of the Palworld client/server being run.

2. **Retrieve Latest Official Release**:
   - Always perform a live web search (e.g., checking SteamDB, Steam News, or official Palworld patch logs) to identify the latest officially released version of Palworld.

3. **Compare and Highlight Mismatches**:
   - Compare the user's running version against the latest official version.
   - If the user's version is outdated, notify them of the mismatch and explicitly list key content (such as new Pals, items, islands, or system changes) they are missing.

## Context Window & Performance Optimization Policy

1. **Avoid Full Server Dumps**:
   - For Dedicated Servers or multiplayer worlds, the global save database contains data for all players and guilds.
   - Do not invoke `query_full` (which lists all players, bases, and guilds) unless explicitly requested by the user. It can return massive JSON payloads that exceed context window limits or cause slower response times.

2. **Isolate Queries with Player UID**:
   - Always prefer targeted query tools: `monitor_pals`, `query_progress`, `query_analyzer`, and `query_breeding` using the optional `player_uid` argument.
   - If the player's UID is unknown, perform a quick initial query or check their local save game filenames (`Players/<PlayerUID>.sav`) to find it.

## Troubleshooting & Decompression Support Policy

1. **Resolve Oodle DLL Missing Errors**:
   - The save parser depends on `oo2core_9_win64.dll` for decompressed memory signature scanning.
   - If the tools return a decompression failure or missing DLL error, guide the user to copy `oo2core_9_win64.dll` from their Palworld game directory (typically under `SteamApps/common/Palworld/Binaries/Win64/`) and place it next to their compiled `palagent-ai.exe` executable or in their user path.

2. **Multiple Save Files Conflict**:
   - If telemetry lists multiple detected game saves or fails to select one, instruct the user to run `palagent-ai.exe --list-worlds` to see all available saves (which displays the human-readable world name read from `LevelMeta.sav` alongside the folder GUID and modification date) or select one interactively using the `--select-world` flag.

## Community Advice & Stacking Glitch Policy

1. **Search Reddit and Steam Guides**:
   - When the user asks for design recommendations, base optimizations, or gameplay efficiency (e.g., *"How can I make my farming/planting more efficient?"*), do not limit your searches to official wikis or static databases.
   - Actively search Reddit, YouTube tutorials, and Steam Community Guides using terms like `"Reddit tips"`, `"glitch"`, `"trick"`, or `"base layout"`.

2. **Present Advanced/Unorthodox Techniques**:
   - Offer advanced community tricks alongside standard gameplay advice.
   - For example, if asked about crop efficiency, explain the community trick of using temporary wooden benches/stools to stack 3 or 4 plantation plots vertically in the exact same footprint to save space, while also warning them about Pal AI pathing limitations if stacked too high.

3. **Avoid Raw Links & Cite Platform/Community Instead**:
   - Do NOT output direct raw links to specific forum threads, Reddit comment IDs, or unverified third-party websites, as they can break, decay, or redirect to inappropriate content.
   - Instead, state clearly the platform, subreddit, or community name where the information was sourced (e.g., *"Sourced from the Palworld community on Reddit"* or *"Based on Steam Community Guides"*).

## MCP Server Tools Reference

The PalAgent AI MCP server exposes the following tools:

1. **`query_time`**:
   - Description: Get current in-game day, time, and cycle (day/night).
   - Parameters: None.

2. **`query_settings`**:
   - Description: Get server configuration and game difficulty settings.
   - Parameters: None.

3. **`chest_search`**:
   - Description: Locate specific items across all base chests.
   - Parameters:
     - `query` (string, required): Item name to search (e.g. "Berries", "Wood").

4. **`query_breeding`**:
   - Description: Analyze available gender combos and potential breeding offspring.
   - Parameters:
     - `player_uid` (string, optional): Optional Player UID to isolate breeding team.

5. **`query_progress`**:
   - Description: Check player notes found, fast travel unlocks, and capture progress.
   - Parameters:
     - `player_uid` (string, optional): Optional Player UID to isolate progress.

6. **`monitor_pals`**:
   - Description: Get real-time sanity (SAN), satiety (hunger), and HP levels of base/active Pals.
   - Parameters:
     - `player_uid` (string, optional): Optional Player UID to isolate monitored Pals.

7. **`query_analyzer`**:
   - Description: Analyze Pal talent IV stats (HP/Atk/Def bonuses) and passive skills.
   - Parameters:
     - `player_uid` (string, optional): Optional Player UID to isolate Pals.

8. **`query_full`**:
   - Description: Retrieve the complete world telemetry report including bases, players, and guilds.
   - Parameters:
     - `player_uid` (string, optional): Optional Player UID to isolate report details.

## CLI & Interactive Console Reference

If running via CLI or the Interactive Console mode, the following commands/flags are supported:

### CLI Options:
- `palagent-ai.exe --list-worlds`: Lists all detected save worlds, displaying their Index, Last Modified date, World Name (read from `LevelMeta.sav`), and Disk Path.
- `palagent-ai.exe --select-world`: Allows selecting a world interactively.
- `palagent-ai.exe --interactive` / `-i`: Launches the stateful memory-cached Interactive Console shell.
- `palagent-ai.exe --json`: Outputs the world analysis report in structured JSON format.

### Interactive Shell Commands:
- `time` / `t`: Get current in-game day and time.
- `settings` / `s`: Get server configuration settings.
- `chest` / `c <item_query>`: Search items across all chests.
- `progress` / `p [uid]`: Get player map and capture progress.
- `monitor` / `m [uid]`: Monitor sanity and health of active Pals.
- `analyzer` / `a [uid] [opts]`: Inspect Pal talent IVs and passives in the Palbox.
  - Options: `--trait <name>`, `--gender <M/F>`, `--min-level <num>`.
- `breeding` / `b [target]`: Calculates breeding offspring or multi-generational breeding path.
- `clean-seeds`: Perform automated seed/trash containers cleaning in-place.
- `export <json/csv> <path>`: Dump parsed state or Pals list to a file.
- `reload`: Reload the save file from disk.
- `help` / `h`: Show available commands help.
- `exit` / `q`: Close interactive shell.
