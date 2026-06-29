use tauri_app_lib::save_editor;
use std::process::Command;
use std::path::Path;

fn detect_python() -> String {
    if Command::new("python").arg("--version").output().is_ok() {
        return "python".to_string();
    }
    if Command::new("python3").arg("--version").output().is_ok() {
        return "python3".to_string();
    }
    "python".to_string()
}

fn main() {
    println!("--- PalSync AI LiveAgent - CLI MVP Test ---");

    // 1. Detect default save path
    let local_appdata = std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA env var not found");
    let save_dir = Path::new(&local_appdata)
        .join("Pal")
        .join("Saved")
        .join("SaveGames");

    if !save_dir.exists() {
        println!("Error: No se encontró la carpeta de guardados de Palworld en la ruta por defecto.");
        return;
    }

    // Find the latest modified world folder
    let mut worlds = Vec::new();
    for entry in std::fs::read_dir(&save_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            // Find Steam ID folder
            for sub_entry in std::fs::read_dir(&path).unwrap() {
                let sub_entry = sub_entry.unwrap();
                let sub_path = sub_entry.path();
                if sub_path.is_dir() && sub_path.file_name().unwrap() != "Cloud" {
                    worlds.push(sub_path);
                }
            }
        }
    }

    if worlds.is_empty() {
        println!("Error: No se encontraron carpetas de mundos guardados.");
        return;
    }

    // Sort by modification time to get the latest
    worlds.sort_by_key(|w| std::fs::metadata(w).unwrap().modified().unwrap());
    let latest_world = worlds.last().unwrap();
    println!("Partida más reciente detectada: {}", latest_world.display());

    let level_sav = latest_world.join("Level.sav");
    let level_json = latest_world.join("Level.tmp.json");
    let py = detect_python();

    println!("Convertiendo Level.sav a JSON...");
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
        .unwrap();

    if !output.status.success() {
        println!("Error en conversión: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }

    // Convert host player save (00000000000000000000000000000001.sav)
    let host_sav = latest_world.join("Players").join("00000000000000000000000000000001.sav");
    let host_json = latest_world.join("Players").join("00000000000000000000000000000001.json");

    if !host_sav.exists() {
        println!("Error: No se encontró el archivo del jugador host.");
        return;
    }

    println!("Convertiendo datos del jugador host...");
    let output = Command::new(&py)
        .args(&[
            "-m",
            "palworld_save_tools.commands.convert",
            host_sav.to_str().unwrap(),
            "--output",
            host_json.to_str().unwrap(),
            "--force",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        println!("Error en conversión de jugador: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }

    // Scan players to get the mapping
    let players = save_editor::scan_players(latest_world.to_str().unwrap(), level_json.to_str().unwrap()).unwrap();
    if players.is_empty() {
        println!("Error: No se pudieron leer los perfiles de jugadores.");
        return;
    }

    let host_player = &players[0];
    println!("Jugador Host detectado: {} (UID: {})", host_player.nickname, host_player.player_uid);

    // List items
    println!("\nInventario común actual:");
    let items = save_editor::get_container_items(level_json.to_str().unwrap(), &host_player.common_container_id).unwrap();
    for item in &items {
        println!("  Slot {}: {} (x{})", item.slot_index + 1, item.item_id, item.count);
    }

    // Modify: Add 99 wood in the first slot to test
    println!("\n[TEST] Modificando slot 0 (primer slot) a 'wood' con cantidad 99...");
    save_editor::modify_container_item(
        level_json.to_str().unwrap(),
        &host_player.common_container_id,
        0,
        "wood",
        99,
    ).unwrap();

    println!("Modificación aplicada en JSON.");

    // List items again
    println!("\nInventario común modificado:");
    let items_mod = save_editor::get_container_items(level_json.to_str().unwrap(), &host_player.common_container_id).unwrap();
    for item in &items_mod {
        println!("  Slot {}: {} (x{})", item.slot_index + 1, item.item_id, item.count);
    }

    // Re-convert Level.tmp.json to Level.sav
    println!("\nGuardando cambios de vuelta en Level.sav...");
    // Backup
    let level_bak = latest_world.join("Level.sav.bak");
    std::fs::copy(&level_sav, &level_bak).unwrap();

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
        .unwrap();

    if !output.status.success() {
        println!("Error al escribir Level.sav: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }

    // Clean up
    std::fs::remove_file(level_json).unwrap();
    std::fs::remove_file(host_json).unwrap();

    println!("\n¡Test completado con éxito! Se modificó tu partida local.");
}
