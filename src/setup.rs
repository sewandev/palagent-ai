use std::path::{Path, PathBuf};

pub enum McpFormat {
    McpServers,
    Servers,
    Opencode,
}

pub fn inject_json_mcp(path: &Path, format: McpFormat, command: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut mcp_config = if path.exists() {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str::<serde_json::Value>(&content)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !mcp_config.is_object() {
        mcp_config = serde_json::json!({});
    }

    if let Some(obj) = mcp_config.as_object_mut() {
        match format {
            McpFormat::Opencode => {
                let mcp_servers = obj.entry("mcp").or_insert_with(|| serde_json::json!({}));
                if let Some(servers_obj) = mcp_servers.as_object_mut() {
                    servers_obj.insert(
                        "palagent-ai".to_string(),
                        serde_json::json!({
                            "type": "local",
                            "command": [command, "mcp"],
                            "enabled": true
                        }),
                    );
                }
            }
            McpFormat::Servers => {
                let mcp_servers = obj
                    .entry("servers")
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(servers_obj) = mcp_servers.as_object_mut() {
                    servers_obj.insert(
                        "palagent-ai".to_string(),
                        serde_json::json!({
                            "type": "stdio",
                            "command": command,
                            "args": ["mcp"]
                        }),
                    );
                }
            }
            McpFormat::McpServers => {
                let mcp_servers = obj
                    .entry("mcpServers")
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(servers_obj) = mcp_servers.as_object_mut() {
                    servers_obj.insert(
                        "palagent-ai".to_string(),
                        serde_json::json!({
                            "command": command,
                            "args": ["mcp"]
                        }),
                    );
                }
            }
        }
    }

    if let Ok(json_str) = serde_json::to_string_pretty(&mcp_config) {
        let _ = std::fs::write(path, json_str);
    }
}

pub fn inject_marker_block(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut file_content = if path.exists() {
        std::fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    let begin_marker = "<!-- BEGIN PALAGENT-AI RULES — managed by palagent-ai setup -->";
    let end_marker = "<!-- END PALAGENT-AI RULES -->";

    let block = format!("{}\n{}\n{}", begin_marker, content.trim(), end_marker);

    if file_content.contains(begin_marker) {
        if let Some(start) = file_content.find(begin_marker) {
            if let Some(end) = file_content.find(end_marker) {
                let actual_end = end + end_marker.len();
                file_content.replace_range(start..actual_end, &block);
            }
        }
    } else {
        if !file_content.is_empty() && !file_content.ends_with('\n') {
            file_content.push('\n');
        }
        file_content.push_str(&block);
        file_content.push('\n');
    }

    let _ = std::fs::write(path, file_content);
}

pub fn print_setup_complete(
    name: &str,
    dest_exe: &Path,
    mcp_config: &Path,
    rules_file: &Path,
    skill_file: Option<&Path>,
) {
    println!("==================================================");
    println!("   PALAGENT-AI {} SETUP COMPLETED", name.to_uppercase());
    println!("==================================================");
    println!(" Permanent Exe: {}", dest_exe.display());
    println!(" MCP Config   : {}", mcp_config.display());
    println!(" Rules File   : {}", rules_file.display());
    if let Some(sf) = skill_file {
        println!(" Skill File   : {}", sf.display());
    }
    println!("==================================================");
}

pub fn run_setup(agent_slug: &str) {
    crate::utils::log_message("INFO", &format!("Starting setup for agent: {}", agent_slug));
    let home_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "C:\\".to_string());
    let home_path = Path::new(&home_dir);

    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("palagent-ai.exe"));

    let palagent_dir = home_path.join(".palagent-ai");
    if let Err(e) = std::fs::create_dir_all(&palagent_dir) {
        println!("Error creating PalAgent directory: {}", e);
        std::process::exit(1);
    }

    let dest_exe = palagent_dir.join("palagent-ai.exe");
    if current_exe != dest_exe {
        if let Err(e) = std::fs::copy(&current_exe, &dest_exe) {
            let warn_msg = format!("Could not copy executable to permanent folder: {}", e);
            crate::utils::log_message("WARNING", &warn_msg);
            println!("Warning: {}", warn_msg);
        } else {
            let log_msg = format!(
                "Copied executable to permanent location: {}",
                dest_exe.display()
            );
            crate::utils::log_message("INFO", &log_msg);
            println!("{}", log_msg);
        }
    }

    let standard_dll_paths = [
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "C:\\Program Files\\Steam\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "D:\\SteamLibrary\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "E:\\SteamLibrary\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "F:\\SteamLibrary\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
    ];

    let mut found_dll_path = None;
    if let Some(parent) = current_exe.parent() {
        let paths_to_check = [
            parent.join("oo2core_9_win64.dll"),
            parent.join("..").join("oo2core_9_win64.dll"),
            parent.join("..").join("..").join("oo2core_9_win64.dll"),
        ];
        for path in &paths_to_check {
            if path.exists() {
                found_dll_path = Some(path.to_path_buf());
                break;
            }
        }
    }
    if found_dll_path.is_none() {
        for path_str in &standard_dll_paths {
            let path = Path::new(path_str);
            if path.exists() {
                found_dll_path = Some(path.to_path_buf());
                break;
            }
        }
    }

    if let Some(dll_path) = found_dll_path {
        let dest_dll = palagent_dir.join("oo2core_9_win64.dll");
        if let Err(e) = std::fs::copy(&dll_path, &dest_dll) {
            let warn_msg = format!(
                "Could not copy oo2core_9_win64.dll to default folder: {}",
                e
            );
            crate::utils::log_message("WARNING", &warn_msg);
            println!("Warning: {}", warn_msg);
        } else {
            let log_msg = format!(
                "Copied oo2core_9_win64.dll to permanent location: {}",
                dest_dll.display()
            );
            crate::utils::log_message("INFO", &log_msg);
            println!("{}", log_msg);
        }
    } else {
        let warn_msg =
            "oo2core_9_win64.dll not found in standard paths. You might need to place it manually."
                .to_string();
        crate::utils::log_message("WARNING", &warn_msg);
        println!("Warning: {}", warn_msg);
    }

    let command_str = dest_exe.to_string_lossy().replace("\\", "/");

    let rule_content = r#"
# PalAgent AI Rules
You have access to PalAgent AI telemetry and monitor tools via MCP.
When the user asks about Palworld save files, in-game stats, Pals, inventory, bases, or breeding, use the palagent-ai MCP tools to retrieve real-time data instead of guessing.
Always verify the game version and perform live web searches on palworld.gg or wikis to keep your information up-to-date.
"#;

    let skill_body = r#"---
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

## Community Advice & Stacking Glitch Policy

1. **Search Reddit and Steam Guides**:
   - When the user asks for design recommendations, base optimizations, or gameplay efficiency (e.g., *"How can I make my farming/planting more efficient?"*), do not limit your searches to official wikis or static databases.
   - Actively search Reddit, YouTube tutorials, and Steam Community Guides using terms like `"Reddit tips"`, `"glitch"`, `"trick"`, or `"base layout"`.

2. **Present Advanced/Unorthodox Techniques**:
   - Offer advanced community tricks alongside standard gameplay advice.
   - For example, if asked about crop efficiency, explain the community trick of using temporary wooden benches/stools to stack 3 or 4 plantation plots vertically in the exact same footprint to save space, while also warning them about Pal AI pathing limitations if stacked too high.

3. **Cite Sources & Direct URLs**:
   - Always provide the direct URLs of the Reddit threads, Steam guides, or web pages you retrieved during your searches.
   - This allows the user to directly verify the source of any advanced community tricks or glitches.
"#;

    match agent_slug {
        "antigravity-cli" => {
            let gemini_config_dir = home_path.join(".gemini").join("config");
            std::fs::create_dir_all(&gemini_config_dir).ok();

            let mcp_config_path = gemini_config_dir.join("mcp_config.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let agents_md_path = gemini_config_dir.join("AGENTS.md");
            inject_marker_block(&agents_md_path, rule_content);

            let gemini_md_path = home_path.join(".gemini").join("GEMINI.md");
            inject_marker_block(&gemini_md_path, rule_content);

            let skill_dir = gemini_config_dir.join("skills").join("palagent-ai");
            std::fs::create_dir_all(&skill_dir).ok();
            std::fs::write(skill_dir.join("SKILL.md"), skill_body).ok();

            print_setup_complete(
                "Antigravity CLI",
                &dest_exe,
                &mcp_config_path,
                &agents_md_path,
                Some(&skill_dir.join("SKILL.md")),
            );
        }
        "vscode-copilot" => {
            let app_data = std::env::var("APPDATA")
                .unwrap_or_else(|_| format!("{}\\AppData\\Roaming", home_dir));
            let vscode_user_dir = Path::new(&app_data).join("Code").join("User");
            std::fs::create_dir_all(&vscode_user_dir).ok();

            let mcp_config_path = vscode_user_dir.join("mcp.json");
            inject_json_mcp(&mcp_config_path, McpFormat::Servers, &command_str);

            let prompts_dir = vscode_user_dir.join("prompts");
            std::fs::create_dir_all(&prompts_dir).ok();
            let instr_file = prompts_dir.join("palagent-ai.instructions.md");
            let copilot_body = format!("---\napplyTo: \"**\"\n---\n\n{}", rule_content);
            std::fs::write(&instr_file, copilot_body).ok();

            print_setup_complete(
                "VS Code Copilot",
                &dest_exe,
                &mcp_config_path,
                &instr_file,
                None,
            );
        }
        "cursor" => {
            let app_data = std::env::var("APPDATA")
                .unwrap_or_else(|_| format!("{}\\AppData\\Roaming", home_dir));
            let cursor_dir = Path::new(&app_data).join("Cursor").join("User");
            std::fs::create_dir_all(&cursor_dir).ok();

            let mcp_config_path = cursor_dir.join("mcp.json");
            inject_json_mcp(&mcp_config_path, McpFormat::Servers, &command_str);

            let rule_file = cursor_dir.join("palagent-ai-rules.md");
            let cursor_rules_body = format!("---\nalwaysApply: true\n---\n\n{}", rule_content);
            std::fs::write(&rule_file, cursor_rules_body).ok();

            print_setup_complete("Cursor", &dest_exe, &mcp_config_path, &rule_file, None);
        }
        "windsurf" => {
            let codeium_dir = home_path.join(".codeium").join("windsurf");
            std::fs::create_dir_all(&codeium_dir).ok();

            let mcp_config_path = codeium_dir.join("mcp_config.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let memories_dir = codeium_dir.join("memories");
            std::fs::create_dir_all(&memories_dir).ok();
            let rules_file = memories_dir.join("palagent-ai-rules.md");
            std::fs::write(&rules_file, rule_content).ok();

            print_setup_complete("Windsurf", &dest_exe, &mcp_config_path, &rules_file, None);
        }
        "opencode" => {
            let config_dir = home_path.join(".config").join("opencode");
            std::fs::create_dir_all(&config_dir).ok();

            let mcp_config_path = config_dir.join("opencode.json");
            inject_json_mcp(&mcp_config_path, McpFormat::Opencode, &command_str);

            let rules_file = config_dir.join("AGENTS.md");
            inject_marker_block(&rules_file, rule_content);

            print_setup_complete("OpenCode", &dest_exe, &mcp_config_path, &rules_file, None);
        }
        "claude-code" => {
            let claude_dir = home_path.join(".claude");
            std::fs::create_dir_all(&claude_dir).ok();

            let mcp_config_path = claude_dir.join("settings.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            print_setup_complete(
                "Claude Code",
                &dest_exe,
                &mcp_config_path,
                &mcp_config_path,
                None,
            );
        }
        "gemini-cli" => {
            let gemini_dir = home_path.join(".gemini");
            std::fs::create_dir_all(&gemini_dir).ok();

            let mcp_config_path = gemini_dir.join("mcp_config.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let system_md = gemini_dir.join("GEMINI.md");
            inject_marker_block(&system_md, rule_content);

            print_setup_complete("Gemini CLI", &dest_exe, &mcp_config_path, &system_md, None);
        }
        "codex" => {
            let codex_dir = home_path.join(".codex");
            std::fs::create_dir_all(&codex_dir).ok();

            let config_toml_path = codex_dir.join("config.toml");
            let mut content = if config_toml_path.exists() {
                std::fs::read_to_string(&config_toml_path).unwrap_or_default()
            } else {
                String::new()
            };

            if !content.contains("[mcp_servers.palagent-ai]") {
                let toml_append = format!(
                    "\n[mcp_servers.palagent-ai]\ncommand = \"{}\"\nargs = [\"mcp\"]\n",
                    command_str
                );
                content.push_str(&toml_append);
                let _ = std::fs::write(&config_toml_path, content);
            }

            let instr_file = codex_dir.join("palagent-ai-instructions.md");
            std::fs::write(&instr_file, rule_content).ok();

            print_setup_complete("Codex", &dest_exe, &config_toml_path, &instr_file, None);
        }
        "qwen" => {
            let qwen_dir = home_path.join(".qwen");
            std::fs::create_dir_all(&qwen_dir).ok();

            let mcp_config_path = qwen_dir.join("settings.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let qwen_md = qwen_dir.join("QWEN.md");
            inject_marker_block(&qwen_md, rule_content);

            print_setup_complete("Qwen Code", &dest_exe, &mcp_config_path, &qwen_md, None);
        }
        "kiro" => {
            let kiro_dir = home_path.join(".kiro");
            std::fs::create_dir_all(&kiro_dir).ok();

            let mcp_config_path = kiro_dir.join("settings").join("mcp.json");
            std::fs::create_dir_all(kiro_dir.join("settings")).ok();
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let steering_file = kiro_dir.join("steering").join("palagent-ai.md");
            std::fs::create_dir_all(kiro_dir.join("steering")).ok();
            std::fs::write(&steering_file, rule_content).ok();

            print_setup_complete(
                "Kiro IDE",
                &dest_exe,
                &mcp_config_path,
                &steering_file,
                None,
            );
        }
        "pi" => {
            let pi_dir = home_path.join(".pi").join("config");
            std::fs::create_dir_all(&pi_dir).ok();

            let mcp_config_path = pi_dir.join("mcp.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            print_setup_complete("Pi", &dest_exe, &mcp_config_path, &mcp_config_path, None);
        }
        "kilocode" => {
            let config_dir = home_path.join(".config").join("kilo");
            std::fs::create_dir_all(&config_dir).ok();

            let mcp_config_path = config_dir.join("opencode.json");
            inject_json_mcp(&mcp_config_path, McpFormat::Opencode, &command_str);

            let rules_file = config_dir.join("AGENTS.md");
            inject_marker_block(&rules_file, rule_content);

            print_setup_complete("Kilo Code", &dest_exe, &mcp_config_path, &rules_file, None);
        }
        _ => {
            println!("Error: Unsupported agent slug '{}'.", agent_slug);
            println!("Supported slugs: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode");
            std::process::exit(1);
        }
    }
}
