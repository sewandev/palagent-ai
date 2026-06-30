use crate::i18n;
use std::io::Write;
use std::path::{Path, PathBuf};

pub static BREED_POWER: &[(&str, u32)] = &[
    ("Chikipi", 1500),
    ("Teafant", 1490),
    ("Lamball", 1470),
    ("Mau", 1480),
    ("Cremis", 1455),
    ("Vixy", 1450),
    ("Cattiva", 1460),
    ("Lifmunk", 1430),
    ("Hoocrates", 1420),
    ("Sparkit", 1410),
    ("Foxsparks", 1400),
    ("Depresso", 1390),
    ("Jolthog", 1370),
    ("Pengullet", 1350),
    ("Fuddler", 1210),
    ("Tanzee", 1230),
    ("Daedream", 1220),
    ("Gumoss", 1240),
    ("Melpaca", 1100),
    ("Celaray", 1080),
    ("Rushoar", 1120),
];

pub fn find_child_pal(power_a: u32, power_b: u32) -> (&'static str, u32) {
    let avg_power = (power_a + power_b).div_ceil(2);
    let mut closest_pal = "Unknown";
    let mut min_diff = u32::MAX;
    for &(name, power) in BREED_POWER {
        let diff = (power as i32 - avg_power as i32).unsigned_abs();
        if diff < min_diff {
            min_diff = diff;
            closest_pal = name;
        }
    }
    (closest_pal, avg_power)
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
        println!(
            " [{}] | {} | {}",
            idx + 1,
            datetime.format("%Y-%m-%d %H:%M:%S"),
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
