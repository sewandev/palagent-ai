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
            let old_exe = palagent_dir.join("palagent-ai.exe.old");
            let _ = std::fs::remove_file(&old_exe);
            if std::fs::rename(&dest_exe, &old_exe).is_ok() {
                if let Err(err) = std::fs::copy(&current_exe, &dest_exe) {
                    let warn_msg = format!(
                        "Could not copy executable to permanent folder (even after rename): {}",
                        err
                    );
                    crate::utils::log_message("WARNING", &warn_msg);
                    println!("Warning: {}", warn_msg);
                } else {
                    let log_msg = format!(
                        "Copied executable to permanent location (via hot-reload rename): {}",
                        dest_exe.display()
                    );
                    crate::utils::log_message("INFO", &log_msg);
                    println!("{}", log_msg);
                }
            } else {
                let warn_msg = format!("Could not copy executable to permanent folder: {}", e);
                crate::utils::log_message("WARNING", &warn_msg);
                println!("Warning: {}", warn_msg);
            }
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
When the user asks about Palworld save files, in-game stats, Pals, inventory, bases, or breeding, use the palagent-ai MCP tools to retrieve real-time data immediately instead of guessing.
Do NOT perform web searches for the player's personal save telemetry (which must be retrieved locally via MCP). However, for any gameplay mechanics, items, drop rates, recipes, or patch updates up to v0.7.3.90464, you MUST perform targeted web searches to ensure the information is always up-to-date and accurate, rather than guessing or assuming.
Prioritize local database and MCP telemetry first, but use online searches as a hybrid extension to verify game mechanics.

## Interaction and User Interface
- **Mandatory ask_question Tool**: Whenever you need to ask the user a question with choices, options, confirmation, or clarifications, you MUST ALWAYS use the `ask_question` tool. Do not print plain-text number options (e.g. "1. Option A, 2. Option B") directly in the chat responses, as this fails to leverage the interactive UI modal.
- **Selectable Options**: Ensure the options are formatted cleanly as the user's direct responses and translated to their preferred language.
- **Direct Meta-Focused Style**: Never be patronizing or condescending, but do not coddle the player or agree with incorrect views. If the player's opinion or preference contradicts technical analysis, database values, or game data, state clearly and objectively that the analysis does not support their view, explain the mathematical/technical reasons why, and focus strictly on achieving the optimal game meta.

## 🎮 Palworld Active Version Context
All agent skills, AI prompts, and code analyses MUST assume that the active, target version of Palworld is v0.7.3.90464 (released June 2026).
1. Web Search Exception for Game Mechanics: While you must not perform web searches to retrieve the player's personal save telemetry (which is queried locally via MCP), you MUST perform active, targeted web searches to verify game mechanics, workstation capabilities (e.g. Pal Surgery Table), patch changes, or item utilities under v0.7.3.90464, rather than assuming or guessing based on outdated pre-2026 memory.
2. Do not assume or mention out-of-date information from 2024 versions.

## 🚫 Direct Response & Terminal-Friendly Formatting Policy
When the user asks anything related to Palworld save files, stats, chests, Pals, breeding, or game analysis:
1. Do NOT generate or write markdown files (artifacts) in the user's workspace.
2. You MUST display all information beautifully and directly inside the chat response window itself.
3. DO NOT use markdown elements (such as bold stars **, headers #, italics *, or block code backticks) in your conversational chat replies. The terminal does not render markdown properly.
4. Instead, format tables and lists using clean plain-text ASCII elements (e.g. pipes |, lines of hyphens -, and numbered steps).
"#;

    let skill_body = include_str!("../skills/palagent-ai/SKILL.md");

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
