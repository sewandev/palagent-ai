pub mod save_editor;

use std::process::Command;
use save_editor::{PlayerInfo, InventoryItem};

fn detect_python() -> String {
    if Command::new("python").arg("--version").output().is_ok() {
        return "python".to_string();
    }
    if Command::new("python3").arg("--version").output().is_ok() {
        return "python3".to_string();
    }
    "python".to_string()
}

#[tauri::command]
fn convert_sav_to_json(python_path: &str, sav_path: &str, output_json_path: &str) -> Result<(), String> {
    let py = if python_path == "default" || python_path.is_empty() {
        detect_python()
    } else {
        python_path.to_string()
    };

    let output = Command::new(&py)
        .args(&[
            "-m",
            "palworld_save_tools.commands.convert",
            sav_path,
            "--output",
            output_json_path,
            "--force",
            "--custom-properties",
            ".worldSaveData.ItemContainerSaveData.Value.RawData",
        ])
        .output()
        .map_err(|e| format!("Failed to execute python conversion: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Python script error: {}", err_msg));
    }

    Ok(())
}

#[tauri::command]
fn convert_json_to_sav(python_path: &str, json_path: &str, output_sav_path: &str) -> Result<(), String> {
    let py = if python_path == "default" || python_path.is_empty() {
        detect_python()
    } else {
        python_path.to_string()
    };

    let output = Command::new(&py)
        .args(&[
            "-m",
            "palworld_save_tools.commands.convert",
            json_path,
            "--output",
            output_sav_path,
            "--force",
        ])
        .output()
        .map_err(|e| format!("Failed to execute python conversion: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Python script error: {}", err_msg));
    }

    Ok(())
}

#[tauri::command]
fn get_default_save_dir() -> Result<String, String> {
    if cfg!(target_os = "windows") {
        if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
            let path = std::path::Path::new(&local_appdata)
                .join("Pal")
                .join("Saved")
                .join("SaveGames");
            if path.exists() {
                return Ok(path.to_string_lossy().into_owned());
            }
        }
    }
    Ok("".to_string())
}

#[tauri::command]
fn load_save_file(python_path: &str, save_dir: &str) -> Result<Vec<PlayerInfo>, String> {
    let py = if python_path == "default" || python_path.is_empty() {
        detect_python()
    } else {
        python_path.to_string()
    };

    let save_path = std::path::Path::new(save_dir);
    let level_sav = save_path.join("Level.sav");
    let level_json = save_path.join("Level.tmp.json");

    if !level_sav.exists() {
        return Err("Level.sav not found in the specified directory".to_string());
    }

    // 1. Convert Level.sav to Level.tmp.json
    let output = Command::new(&py)
        .args(&[
            "-m",
            "palworld_save_tools.commands.convert",
            level_sav.to_str().unwrap(),
            "--output",
            level_json.to_str().unwrap(),
            "--force",
            "--custom-properties",
            ".worldSaveData.ItemContainerSaveData.Value.RawData",
        ])
        .output()
        .map_err(|e| format!("Failed to convert Level.sav: {}", e))?;

    if !output.status.success() {
        return Err(format!("Python Level.sav conversion failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // 2. Convert all player .sav files in Players/ directory to .json
    let players_dir = save_path.join("Players");
    if players_dir.exists() {
        for entry in std::fs::read_dir(&players_dir).map_err(|e| format!("Failed to read Players dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read Players entry: {}", e))?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sav") {
                let json_path = path.with_extension("json");
                Command::new(&py)
                    .args(&[
                        "-m",
                        "palworld_save_tools.commands.convert",
                        path.to_str().unwrap(),
                        "--output",
                        json_path.to_str().unwrap(),
                        "--force",
                    ])
                    .output()
                    .map_err(|e| format!("Failed to convert player sav: {}", e))?;
            }
        }
    }

    // 3. Scan players to match them with nicknames
    save_editor::scan_players(save_dir, level_json.to_str().unwrap())
}

#[tauri::command]
fn apply_save_changes(python_path: &str, save_dir: &str) -> Result<(), String> {
    let py = if python_path == "default" || python_path.is_empty() {
        detect_python()
    } else {
        python_path.to_string()
    };

    let save_path = std::path::Path::new(save_dir);
    let level_sav = save_path.join("Level.sav");
    let level_bak = save_path.join("Level.sav.bak");
    let level_json = save_path.join("Level.tmp.json");

    if !level_json.exists() {
        return Err("No modified Level.tmp.json found. Load save first.".to_string());
    }

    // 1. Create backup of Level.sav
    if level_sav.exists() {
        std::fs::copy(&level_sav, &level_bak)
            .map_err(|e| format!("Failed to create backup Level.sav.bak: {}", e))?;
    }

    // 2. Convert Level.tmp.json back to Level.sav
    let output = Command::new(&py)
        .args(&[
            "-m",
            "palworld_save_tools.commands.convert",
            level_json.to_str().unwrap(),
            "--output",
            level_sav.to_str().unwrap(),
            "--force",
        ])
        .output()
        .map_err(|e| format!("Failed to write Level.sav: {}", e))?;

    if !output.status.success() {
        return Err(format!("Python Level.sav write failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // 3. Clean up temporary files
    if level_json.exists() {
        let _ = std::fs::remove_file(&level_json);
    }
    let players_dir = save_path.join("Players");
    if players_dir.exists() {
        for entry in std::fs::read_dir(&players_dir).map_err(|e| format!("Failed to read Players dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read Players entry: {}", e))?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn scan_players(save_dir: &str, level_json_path: &str) -> Result<Vec<PlayerInfo>, String> {
    save_editor::scan_players(save_dir, level_json_path)
}

#[tauri::command]
fn get_container_items(level_json_path: &str, container_guid: &str) -> Result<Vec<InventoryItem>, String> {
    save_editor::get_container_items(level_json_path, container_guid)
}

#[tauri::command]
fn modify_container_item(
    level_json_path: &str,
    container_guid: &str,
    slot_index: usize,
    item_id: &str,
    count: u32,
) -> Result<(), String> {
    save_editor::modify_container_item(level_json_path, container_guid, slot_index, item_id, count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            convert_sav_to_json,
            convert_json_to_sav,
            get_default_save_dir,
            load_save_file,
            apply_save_changes,
            scan_players,
            get_container_items,
            modify_container_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

