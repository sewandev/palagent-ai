use crate::i18n;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn find_child_pal(parent_a: &str, parent_b: &str) -> (String, u32) {
    // 1. Check for breeding exceptions
    if let Some(child_id) = crate::db::check_breeding_exception(parent_a, parent_b) {
        let use_es = i18n::current_language() == i18n::Language::Es;
        let child_translated = crate::db::translate_pal(&child_id, use_es);
        return (child_translated, 0);
    }

    // 2. Query breed power dynamically
    let power_a = crate::db::get_breed_power_by_name(parent_a);
    let power_b = crate::db::get_breed_power_by_name(parent_b);
    let avg_power = (power_a + power_b + 1) / 2;

    // 3. Find closest pal by power
    let child_id = crate::db::find_closest_pal_by_breed_power(avg_power);
    let use_es = i18n::current_language() == i18n::Language::Es;
    let child_translated = crate::db::translate_pal(&child_id, use_es);
    (child_translated, avg_power as u32)
}

pub fn get_all_detected_worlds() -> Vec<(PathBuf, std::time::SystemTime)> {
    let mut valid_worlds = Vec::new();
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let path = std::path::Path::new(&local_appdata)
            .join("Pal")
            .join("Saved")
            .join("SaveGames");
        if path.exists() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let user_path = entry.path();
                    if user_path.is_dir() {
                        if let Ok(sub_entries) = std::fs::read_dir(&user_path) {
                            for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                                let world_path = sub_entry.path();
                                if world_path.is_dir() && world_path.file_name().unwrap() != "Cloud"
                                {
                                    let level_sav = world_path.join("Level.sav");
                                    if level_sav.exists() {
                                        if let Ok(meta) = std::fs::metadata(&level_sav) {
                                            if let Ok(modified) = meta.modified() {
                                                valid_worlds.push((world_path, modified));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    valid_worlds.sort_by_key(|b| std::cmp::Reverse(b.1));
    valid_worlds
}

pub fn select_world_interactively(worlds: &[(PathBuf, std::time::SystemTime)]) -> PathBuf {
    println!("\n=== {} ===\n", i18n::t("list_worlds_title"));
    println!(" {}", i18n::t("list_worlds_header"));
    println!("{}", "-".repeat(80));
    for (idx, (path, modified)) in worlds.iter().enumerate() {
        let datetime: chrono::DateTime<chrono::Local> = (*modified).into();
        let world_name = get_world_name(path);
        let game_mode_key = detect_game_mode(path);
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
    println!("{}", "-".repeat(80));

    let prompt = i18n::t("select_world_prompt").replace("{}", &worlds.len().to_string());
    print!("{}", prompt);
    let _ = std::io::stdout().flush();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice >= 1 && choice <= worlds.len() {
                return worlds[choice - 1].0.clone();
            }
        }
    }

    println!("{}", i18n::t("invalid_selection"));
    worlds[0].0.clone()
}

pub fn detect_game_mode(world_path: &Path) -> String {
    let players_dir = world_path.join("Players");
    if !players_dir.exists() {
        return "game_mode_singleplayer".to_string();
    }

    let mut has_host = false;
    let mut has_others = false;

    if let Ok(entries) = std::fs::read_dir(&players_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sav") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem == "00000000000000000000000000000001" {
                        has_host = true;
                    } else if stem.len() == 32 {
                        has_others = true;
                    }
                }
            }
        }
    }

    if has_host && has_others {
        "game_mode_coop".to_string()
    } else if has_host {
        "game_mode_singleplayer".to_string()
    } else if has_others {
        "game_mode_dedicated".to_string()
    } else {
        "game_mode_singleplayer".to_string()
    }
}

pub fn log_message(level: &str, message: &str) {
    let home_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "C:\\".to_string());
    let log_dir = Path::new(&home_dir).join(".palagent-ai");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_file_path = log_dir.join("palagent.log");

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] [{}] {}", timestamp, level, message);
    }
}

pub fn detect_local_player_uid() -> Result<(String, u64), String> {
    let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
        format!("{}\\AppData\\Local", home)
    });
    let save_dir = std::path::Path::new(&local_appdata)
        .join("Pal")
        .join("Saved")
        .join("SaveGames");

    if !save_dir.exists() {
        return Err(format!(
            "No se encontró el directorio de guardado de Palworld en: {}",
            save_dir.display()
        ));
    }

    let mut steam_ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&save_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name.len() == 17 && name.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(steam_id) = name.parse::<u64>() {
                            if steam_id >= 76561197960265728 {
                                let modified = path
                                    .metadata()
                                    .and_then(|m| m.modified())
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                                steam_ids.push((steam_id, modified));
                            }
                        }
                    }
                }
            }
        }
    }

    if steam_ids.is_empty() {
        return Err("No se encontraron carpetas con SteamID válidos.".to_string());
    }

    steam_ids.sort_by_key(|b| std::cmp::Reverse(b.1));
    let steam_id = steam_ids[0].0;

    let account_id = steam_id - 76561197960265728;
    let hex_str = format!("{:08X}", account_id);
    let guid = format!("{}-0000-0000-0000-000000000000", hex_str);

    Ok((guid, steam_id))
}

pub fn get_world_name(world_path: &Path) -> String {
    let level_meta_sav = world_path.join("LevelMeta.sav");
    if level_meta_sav.exists() {
        if let Ok(bytes) = crate::decompress::decompress_gvas(&level_meta_sav) {
            let name = crate::scanner::extract_string_prop(&bytes, b"WorldName\x00");
            if !name.is_empty() {
                return name;
            }
        }
    }
    // Fallback to the folder name (GUID)
    world_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| i18n::t("unknown"))
}
