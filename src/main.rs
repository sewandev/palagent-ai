macro_rules! print {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        let captured = crate::output::OUTPUT_BUFFER.with(|buf| {
            let mut b = buf.borrow_mut();
            if let Some(ref mut s) = *b {
                s.push_str(&msg);
                true
            } else {
                false
            }
        });
        if !captured {
            use std::io::Write;
            let _ = std::io::stdout().write_all(msg.as_bytes());
        }
    }};
}

macro_rules! println {
    () => {
        print!("\n")
    };
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        let captured = crate::output::OUTPUT_BUFFER.with(|buf| {
            let mut b = buf.borrow_mut();
            if let Some(ref mut s) = *b {
                s.push_str(&msg);
                s.push('\n');
                true
            } else {
                false
            }
        });
        if !captured {
            use std::io::Write;
            let _ = std::io::stdout().write_all(msg.as_bytes());
            let _ = std::io::stdout().write_all(b"\n");
        }
    }};
}

mod commands;
mod db;
mod decompress;
mod i18n;
mod interactive;
mod mcp;
mod models;
mod output;
mod scanner;
mod server;
mod setup;
mod utils;

use crate::commands::{
    run_analyzer_command, run_breeding_command, run_clean_seeds_command, run_full_command,
    run_monitor_command, run_progress_command, run_search_chest_command, run_settings_command,
    run_time_command,
};
use crate::mcp::run_mcp_loop;
use crate::server::run_client_request;
use crate::server::start_host_server;
use crate::setup::run_setup;
use crate::utils::{get_all_detected_worlds, select_world_interactively};
use std::path::PathBuf;

fn main() {
    i18n::init(i18n::detect_system_language());
    crate::db::init_database();

    let all_args: Vec<String> = std::env::args().collect();
    crate::utils::log_message(
        "INFO",
        &format!("Executing palagent-ai with args: {:?}", all_args),
    );

    let args_list: Vec<String> = std::env::args().skip(1).collect();
    let is_json = args_list.iter().any(|arg| arg == "--json");

    let setup_pos = args_list.iter().position(|arg| {
        let a = arg.trim().to_lowercase();
        a == "setup"
    });
    let mut setup_agent = None;
    if let Some(pos) = setup_pos {
        if pos + 1 < args_list.len() {
            setup_agent = Some(args_list[pos + 1].clone());
        }
    }

    if setup_agent.is_none()
        && args_list.iter().any(|arg| {
            let a = arg.trim().to_lowercase();
            a == "--setup-antigravity"
        })
    {
        setup_agent = Some("antigravity-cli".to_string());
    }

    let has_mcp = args_list.iter().any(|arg| {
        let a = arg.trim().to_lowercase();
        a == "mcp" || a == "--mcp"
    });

    if let Some(ref agent) = setup_agent {
        run_setup(agent);
        std::process::exit(0);
    }

    let mut world_path_arg = None;
    let mut skip_next = false;
    for arg in &args_list {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--search-chest"
            || arg == "-c"
            || arg == "--connect"
            || arg == "--passcode"
            || arg == "--player-uid"
            || arg == "--uid"
            || arg == "--port"
        {
            skip_next = true;
            continue;
        }
        let arg_clean = arg.trim().to_lowercase();
        if !arg.starts_with("-")
            && arg_clean != "mcp"
            && arg_clean != "host"
            && arg_clean != "setup"
            && arg_clean != "local-uid"
            && arg_clean != "interactive"
        {
            world_path_arg = Some(arg.clone());
            break;
        }
    }

    let has_time = args_list.iter().any(|arg| arg == "--time" || arg == "-t");
    let has_settings = args_list
        .iter()
        .any(|arg| arg == "--settings" || arg == "-s");
    let has_breeding = args_list
        .iter()
        .any(|arg| arg == "--breeding" || arg == "-b");
    let has_progress = args_list
        .iter()
        .any(|arg| arg == "--progress" || arg == "-p");
    let has_clean_seeds = args_list.iter().any(|arg| arg == "--clean-seeds");
    let has_monitor = args_list
        .iter()
        .any(|arg| arg == "--monitor" || arg == "-m");
    let has_analyzer = args_list
        .iter()
        .any(|arg| arg == "--analyzer" || arg == "-a");
    let has_list_worlds = args_list
        .iter()
        .any(|arg| arg == "--list-worlds" || arg == "-l");
    let has_select_world = args_list.iter().any(|arg| arg == "--select-world");
    let has_host = args_list.iter().any(|arg| arg == "--host");
    let has_local_uid = args_list
        .iter()
        .any(|arg| arg == "local-uid" || arg == "--local-uid");
    let has_interactive = args_list
        .iter()
        .any(|arg| arg == "--interactive" || arg == "-i");

    let mut connect_arg = None;
    if let Some(pos) = args_list.iter().position(|arg| arg == "--connect") {
        if pos + 1 < args_list.len() {
            connect_arg = Some(args_list[pos + 1].clone());
        }
    }

    let mut passcode_arg = None;
    if let Some(pos) = args_list.iter().position(|arg| arg == "--passcode") {
        if pos + 1 < args_list.len() {
            passcode_arg = Some(args_list[pos + 1].clone());
        }
    }

    let mut player_uid_arg = None;
    if let Some(pos) = args_list
        .iter()
        .position(|arg| arg == "--player-uid" || arg == "--uid")
    {
        if pos + 1 < args_list.len() {
            player_uid_arg = Some(args_list[pos + 1].clone());
        }
    }

    // Autodetect player UID if not provided and in client mode
    if player_uid_arg.is_none() && connect_arg.is_some() {
        if let Ok((guid, _)) = crate::utils::detect_local_player_uid() {
            player_uid_arg = Some(guid);
        }
    }

    let mut port_val = 8212;
    if let Some(pos) = args_list.iter().position(|arg| arg == "--port") {
        if pos + 1 < args_list.len() {
            if let Ok(p) = args_list[pos + 1].parse::<u16>() {
                port_val = p;
            }
        }
    }

    let mut search_chest_query = None;
    if let Some(pos) = args_list
        .iter()
        .position(|arg| arg == "--search-chest" || arg == "-c")
    {
        if pos + 1 < args_list.len() {
            search_chest_query = Some(args_list[pos + 1].clone());
        }
    }

    if has_local_uid {
        match crate::utils::detect_local_player_uid() {
            Ok((guid, steam_id)) => {
                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "success",
                            "player_uid": guid,
                            "steam_id": steam_id.to_string()
                        })
                    );
                } else {
                    println!("Player UID : {}", guid);
                    println!("Steam ID   : {}", steam_id);
                }
            }
            Err(e) => {
                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "error",
                            "message": e
                        })
                    );
                } else {
                    println!("Error: {}", e);
                }
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }

    if has_list_worlds {
        let worlds = get_all_detected_worlds();
        if worlds.is_empty() {
            println!("{}", i18n::t("no_worlds_detected"));
            std::process::exit(0);
        }
        println!("\n=== {} ===\n", i18n::t("list_worlds_title"));
        println!(" {}", i18n::t("list_worlds_header"));
        println!("{}", "-".repeat(80));
        for (idx, (path, modified)) in worlds.iter().enumerate() {
            let datetime: chrono::DateTime<chrono::Local> = (*modified).into();
            let world_name = crate::utils::get_world_name(path);
            let game_mode_key = crate::utils::detect_game_mode(path);
            let game_mode = i18n::t(&game_mode_key);
            println!(
                " [{}] | {} | {} | {} | {}",
                idx + 1,
                datetime.format("%Y-%m-%d %H:%M:%S"),
                game_mode,
                world_name,
                path.display()
            );
        }
        println!(
            "\n================================================================================"
        );
        std::process::exit(0);
    }

    // Client Mode connection / MCP Client Mode
    if let Some(ref connect_host) = connect_arg {
        let passcode = passcode_arg.unwrap_or_default();
        if has_mcp {
            run_mcp_loop(None, Some((connect_host.clone(), passcode, player_uid_arg)));
            std::process::exit(0);
        } else {
            let cmd = if has_time {
                "time"
            } else if has_settings {
                "settings"
            } else if let Some(ref q) = search_chest_query {
                &format!("search-chest:{}", q)
            } else if has_breeding {
                "breeding"
            } else if has_progress {
                "progress"
            } else if has_clean_seeds {
                "clean-seeds"
            } else if has_monitor {
                "monitor"
            } else if has_analyzer {
                "analyzer"
            } else {
                "full"
            };

            run_client_request(
                connect_host,
                &passcode,
                cmd,
                is_json,
                player_uid_arg.as_deref(),
            );
            std::process::exit(0);
        }
    }

    // MCP Mode execution (Local)
    if has_mcp {
        let world_path = match world_path_arg {
            Some(ref p) => PathBuf::from(p),
            None => {
                let worlds = get_all_detected_worlds();
                if worlds.is_empty() {
                    let err_json = serde_json::json!({
                        "status": "error",
                        "message": i18n::t("error_detect_save")
                    });
                    println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                    std::process::exit(1);
                }
                if has_select_world {
                    select_world_interactively(&worlds)
                } else {
                    worlds[0].0.clone()
                }
            }
        };
        run_mcp_loop(Some(world_path), None);
        std::process::exit(0);
    }

    // Host Mode execution
    if has_host {
        let passcode = passcode_arg.unwrap_or_else(|| {
            let u = uuid::Uuid::new_v4().to_string();
            u[..6].to_ascii_uppercase()
        });

        let world_path = match world_path_arg {
            Some(ref p) => PathBuf::from(p),
            None => {
                let worlds = get_all_detected_worlds();
                if worlds.is_empty() {
                    let err_json = serde_json::json!({
                        "status": "error",
                        "message": i18n::t("error_detect_save")
                    });
                    println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                    std::process::exit(1);
                }
                if has_select_world {
                    select_world_interactively(&worlds)
                } else {
                    worlds[0].0.clone()
                }
            }
        };

        start_host_server(world_path, port_val, passcode);
        std::process::exit(0);
    }

    // Singleplayer Mode execution
    let world_path = match world_path_arg {
        Some(ref p) => PathBuf::from(p),
        None => {
            let worlds = get_all_detected_worlds();
            if worlds.is_empty() {
                let err_json = serde_json::json!({
                    "status": "error",
                    "message": i18n::t("error_detect_save")
                });
                println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                std::process::exit(1);
            }
            if has_select_world {
                select_world_interactively(&worlds)
            } else {
                worlds[0].0.clone()
            }
        }
    };

    if has_interactive {
        crate::interactive::run_interactive_loop(&world_path, is_json);
        std::process::exit(0);
    }

    if has_time {
        run_time_command(&world_path, is_json);
        std::process::exit(0);
    }
    if has_settings {
        run_settings_command(&world_path, is_json);
        std::process::exit(0);
    }
    if let Some(ref query) = search_chest_query {
        run_search_chest_command(&world_path, query, is_json);
        std::process::exit(0);
    }
    if has_breeding {
        run_breeding_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }
    if has_progress {
        run_progress_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }
    if has_clean_seeds {
        run_clean_seeds_command(&world_path, is_json);
        std::process::exit(0);
    }
    if has_monitor {
        run_monitor_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }
    if has_analyzer {
        run_analyzer_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }

    run_full_command(&world_path, is_json, player_uid_arg.as_deref());
}
