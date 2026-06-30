---
name: palagent-ai
description: Extract telemetry, stats, IVs, breeding combinations, and base camps from Palworld save files.
---

# PalAgent AI Skill

This skill allows the agent to interact with the PalAgent AI MCP server and query real-time Palworld statistics.
Use the `palagent-ai` tools when:
- The user asks for the status of base camps or Palbox.
- The user wants to analyze Pal IVs, stats, or passive skills.
- The user requests breeding combinations.
- The user needs to locate items in base chests.

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
   - If telemetry lists multiple detected game saves or fails to select one, instruct the user to run `palagent-ai.exe --list-worlds` to see all available saves or select one interactively using the `--select-world` flag.
