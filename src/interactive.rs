use crate::commands::run_settings_command;
use crate::decompress::decompress_gvas;
use crate::i18n;
use crate::models::OutputJson;
use crate::scanner::{
    clean_seeds_in_bytes, compress_and_write_gvas, extract_array_strings, extract_bool_prop,
    extract_byte_prop, extract_fixed_point_prop, extract_float_prop, extract_guid_prop,
    extract_int64_prop, extract_int_prop, extract_string_prop, find_chest_containers,
    parse_container_items, scan_base_camps, scan_character_save_parameters, scan_guilds,
};
use crate::utils::{detect_game_mode, find_child_pal, BREED_POWER};
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::path::Path;

fn get_player_uid(world_path: &Path, target_uid: Option<&str>) -> String {
    if let Some(uid) = target_uid {
        return uid.to_string();
    }
    if let Ok((guid, _)) = crate::utils::detect_local_player_uid() {
        return guid;
    }
    let players_dir = world_path.join("Players");
    if let Ok(entries) = std::fs::read_dir(&players_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sav") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem != "00000000000000000000000000000001" && stem.len() == 32 {
                        let mut guid_str = stem.to_string();
                        guid_str.insert(8, '-');
                        guid_str.insert(13, '-');
                        guid_str.insert(18, '-');
                        guid_str.insert(23, '-');
                        return guid_str;
                    }
                }
            }
        }
    }
    "00000000-0000-0000-0000-000000000001".to_string()
}

pub fn run_interactive_loop(world_path: &Path, _is_json: bool) {
    println!("==================================================");
    println!("   PalAgent AI - Modo Consola Interactiva");
    println!("==================================================");
    println!("Cargando partida: {}", world_path.display());
    println!("Por favor, espera mientras se descomprime Level.sav...");

    let level_sav = world_path.join("Level.sav");
    let mut level_bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            println!("Error al descomprimir Level.sav: {}", e);
            return;
        }
    };
    println!("Level.sav cargado con éxito en memoria.");
    println!("Escribe 'help' o 'h' para ver la lista de comandos.");
    println!("==================================================");

    let player_uid = get_player_uid(world_path, None);
    println!("Jugador activo detectado (UID): {}", player_uid);
    println!("--------------------------------------------------");

    loop {
        print!("palagent-ai> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input_trim = input.trim();
        if input_trim.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input_trim.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "help" | "h" => {
                print_help();
            }
            "time" | "t" => {
                handle_time(&level_bytes);
            }
            "settings" | "s" => {
                run_settings_command(world_path, false);
            }
            "chest" | "c" => {
                if parts.len() < 2 {
                    println!("Uso: chest <nombre_objeto>");
                } else {
                    let query = parts[1..].join(" ");
                    handle_chest(&level_bytes, &query);
                }
            }
            "progress" | "p" => {
                let target = parts.get(1).copied().unwrap_or(player_uid.as_str());
                run_progress_command_internal(world_path, target);
            }
            "monitor" | "m" => {
                let target = parts.get(1).copied().unwrap_or(player_uid.as_str());
                handle_monitor(&level_bytes, target);
            }
            "analyzer" | "a" => {
                let mut trait_filter = None;
                let mut gender_filter = None;
                let mut level_filter = None;

                let mut idx = 1;
                let mut target_str = player_uid.clone();
                if idx < parts.len() && !parts[idx].starts_with("--") {
                    target_str = parts[idx].to_string();
                    idx += 1;
                }

                while idx < parts.len() {
                    match parts[idx] {
                        "--trait" => {
                            if idx + 1 < parts.len() {
                                trait_filter = Some(parts[idx + 1]);
                                idx += 2;
                            } else {
                                println!("Error: Falta valor para --trait");
                                idx += 1;
                            }
                        }
                        "--gender" => {
                            if idx + 1 < parts.len() {
                                gender_filter = Some(parts[idx + 1]);
                                idx += 2;
                            } else {
                                println!("Error: Falta valor para --gender");
                                idx += 1;
                            }
                        }
                        "--min-level" => {
                            if idx + 1 < parts.len() {
                                if let Ok(val) = parts[idx + 1].parse::<u32>() {
                                    level_filter = Some(val);
                                }
                                idx += 2;
                            } else {
                                println!("Error: Falta valor para --min-level");
                                idx += 1;
                            }
                        }
                        _ => {
                            println!("Opción no reconocida: {}", parts[idx]);
                            idx += 1;
                        }
                    }
                }

                handle_analyzer(
                    &level_bytes,
                    &target_str,
                    trait_filter,
                    gender_filter,
                    level_filter,
                );
            }
            "breeding" | "b" => {
                let target_pal = if parts.len() >= 2 {
                    Some(parts[1])
                } else {
                    None
                };
                handle_breeding(&level_bytes, &player_uid, target_pal);
            }
            "clean-seeds" => {
                handle_clean_seeds(&mut level_bytes, world_path);
            }
            "export" => {
                if parts.len() < 3 {
                    println!("Uso: export <json/csv> <ruta_archivo>");
                } else {
                    let format = parts[1];
                    let file_path = parts[2];
                    handle_export(&level_bytes, world_path, format, file_path, &player_uid);
                }
            }
            "reload" => {
                println!("Recargando Level.sav desde el disco...");
                match decompress_gvas(&level_sav) {
                    Ok(b) => {
                        level_bytes = b;
                        println!("Level.sav recargado con éxito.");
                    }
                    Err(e) => {
                        println!("Error al recargar Level.sav: {}", e);
                    }
                }
            }
            "exit" | "q" | "quit" => {
                println!("Saliendo del modo interactivo.");
                break;
            }
            _ => {
                println!(
                    "Comando desconocido. Escribe 'help' o 'h' para ver la lista de comandos."
                );
            }
        }
    }
}

fn print_help() {
    println!("\n=== Comandos Disponibles ===");
    println!("  time, t                     Muestra el día y la hora de la partida");
    println!("  settings, s                 Muestra la dificultad y reglas del servidor");
    println!("  chest, c <query>            Busca un objeto en todos los cofres del mundo");
    println!("  progress, p [uid]           Muestra el progreso del jugador");
    println!("  monitor, m [uid]            Muestra la salud y el estrés de los Pals activos");
    println!("  analyzer, a [uid] [opts]    Inspecciona detalladamente los Pals en el Palbox");
    println!("                              Opciones de filtro:");
    println!("                                --trait <nombre>     Filtra por rasgo pasivo");
    println!("                                --gender <M/F>       Filtra por género");
    println!("                                --min-level <num>    Filtra por nivel mínimo");
    println!("  breeding, b [target]        Si se especifica un Pal objetivo, calcula la ruta");
    println!("                              de crianza. Si no, muestra las parejas disponibles.");
    println!("  clean-seeds                 Elimina semillas y basura excedente del inventario");
    println!("  export <json/csv> <path>    Exporta los datos extraídos a un archivo");
    println!("  reload                      Vuelve a leer el archivo de guardado desde el disco");
    println!("  help, h                     Muestra esta ayuda");
    println!("  exit, q                     Sale del modo interactivo");
    println!("============================\n");
}

fn handle_time(level_bytes: &[u8]) {
    let ticks = extract_int64_prop(level_bytes, b"GameDateTimeTicks\x00");
    let ticks_per_day = 864_000_000_000;
    let ticks_per_hour = 36_000_000_000;
    let ticks_per_minute = 600_000_000;

    let total_days = ticks / ticks_per_day;
    let day_number = total_days + 1;
    let remaining_ticks = ticks % ticks_per_day;
    let hour = remaining_ticks / ticks_per_hour;
    let remaining_ticks = remaining_ticks % ticks_per_hour;
    let minute = remaining_ticks / ticks_per_minute;

    let is_day = (6..20).contains(&hour);
    let state = if is_day {
        i18n::t("DIA")
    } else {
        i18n::t("NOCHE")
    };
    println!("\n=== {} ===\n", i18n::t("time_report_title"));
    println!(" {:<35} : {}", i18n::t("ticks_totales"), ticks);
    println!(" {:<35} : {}", i18n::t("dia_juego"), day_number);
    println!(
        " {:<35} : {:02}:{:02}",
        i18n::t("hora_partida"),
        hour,
        minute
    );
    println!(" {:<35} : {}", i18n::t("estado_actual"), state);
    println!("\n=========================================");
}

fn handle_chest(level_bytes: &[u8], query: &str) {
    let query_lower = query.to_lowercase();
    let chests = find_chest_containers(level_bytes);
    let mut found_any = false;
    println!("\n=== {}: '{}' ===\n", i18n::t("chest_search_title"), query);
    println!(
        " {:<25} | {:<5} | {:<25} | {}",
        i18n::t("col_item"),
        i18n::t("col_count"),
        i18n::t("col_chest"),
        i18n::t("col_coords")
    );
    println!("{}", "-".repeat(85));

    for (guid, chest_type, coords) in chests {
        let items = parse_container_items(level_bytes, &guid);
        for item in items {
            let item_name = i18n::t(&item.item_id);
            if item.item_id.to_lowercase().contains(&query_lower)
                || item_name.to_lowercase().contains(&query_lower)
            {
                found_any = true;
                let coords_str = if coords.0 != 0 || coords.1 != 0 {
                    format!("({}, {}, {})", coords.0, coords.1, coords.2)
                } else {
                    i18n::t("no_coords")
                };
                let translated_chest_type = i18n::t(&chest_type);
                println!(
                    " {:<25} | {:<5} | {:<25} | {}",
                    item_name, item.count, translated_chest_type, coords_str
                );
            }
        }
    }
    if !found_any {
        println!(" {}", i18n::t("no_items_found"));
    }
    println!(
        "\n====================================================================================="
    );
}

fn run_progress_command_internal(world_path: &Path, player_uid: &str) {
    let players_dir = world_path.join("Players");
    let stem = player_uid.replace("-", "");
    let path = players_dir.join(format!("{}.sav", stem));
    if !path.exists() {
        println!(
            "No se encontró el archivo de guardado para el jugador: {}",
            player_uid
        );
        return;
    }

    let bytes = match decompress_gvas(&path) {
        Ok(b) => b,
        Err(e) => {
            println!("Error al descomprimir archivo de progreso: {}", e);
            return;
        }
    };

    let relics = crate::scanner::extract_map_keys(&bytes, b"RelicObtainForInstanceFlag\x00");
    let notes = crate::scanner::extract_map_keys(&bytes, b"NoteObtainForInstanceFlag\x00");
    let fast_travels = crate::scanner::extract_map_keys(&bytes, b"FastTravelPointUnlockFlag\x00");
    let areas = crate::scanner::extract_map_keys(&bytes, b"FindAreaFlagMap\x00");
    let captures = crate::scanner::extract_map_counts(&bytes, b"PalCaptureCount\x00");

    println!("\n=== {} ===\n", i18n::t("progress_report_title"));
    println!(" {:<35} : {}", i18n::t("relics_found_label"), relics.len());
    println!(" {:<35} : {}", i18n::t("notes_found_label"), notes.len());
    println!(
        " {:<35} : {}",
        i18n::t("fast_travel_label"),
        fast_travels.len()
    );
    println!(" {:<35} : {}", i18n::t("areas_found_label"), areas.len());
    println!("\n--- {} ---", i18n::t("capture_stats_header"));

    let mut capture_vec: Vec<(String, u32)> = captures.into_iter().collect();
    capture_vec.sort_by_key(|b| std::cmp::Reverse(b.1));

    for (pal_id, count) in capture_vec {
        println!("  * {:<25} : {} / 10", i18n::t(&pal_id), count);
    }
    println!("\n=========================================");
}

fn handle_monitor(level_bytes: &[u8], player_uid: &str) {
    let characters = scan_character_save_parameters(level_bytes);
    println!("\n=== {} ===\n", i18n::t("monitor_report_title"));
    println!(
        " {:<20} | {:<5} | {:<8} | {:<8} | {}",
        i18n::t("col_pal"),
        i18n::t("level"),
        i18n::t("col_health"),
        i18n::t("col_san"),
        i18n::t("col_status")
    );
    println!("{}", "-".repeat(65));

    let mut found_any = false;
    for char_entry in &characters {
        let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
        if is_player {
            continue;
        }

        let owner_uid =
            extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
        if owner_uid != player_uid {
            continue;
        }

        let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
        if char_id.is_empty() || char_id == "Desconocido" {
            continue;
        }

        let level = extract_byte_prop(&char_entry.raw_data, b"Level\x00");
        let level_val = if level == 0 { 1 } else { level };

        let hp_cur = extract_fixed_point_prop(&char_entry.raw_data, b"Hp\x00");
        let hp_max = extract_fixed_point_prop(&char_entry.raw_data, b"MaxHp\x00");
        let hp_pct = if hp_max > 0.0 {
            (hp_cur / hp_max * 100.0) as u32
        } else {
            100
        };

        let mut san_val = 100.0;
        if crate::scanner::has_prop(&char_entry.raw_data, b"SanityValue\x00") {
            san_val = extract_float_prop(&char_entry.raw_data, b"SanityValue\x00");
        }

        let mut satiety = 100.0;
        if crate::scanner::has_prop(&char_entry.raw_data, b"FullStomach\x00") {
            satiety = extract_float_prop(&char_entry.raw_data, b"FullStomach\x00");
        }

        let mut status = "status_excellent".to_string();
        if san_val < 50.0 {
            status = "status_very_stressed".to_string();
        } else if san_val < 70.0 {
            status = "status_stressed".to_string();
        }

        if satiety < 40.0 {
            status = "status_critically_hungry".to_string();
        } else if satiety < 70.0 {
            status = "status_needs_food".to_string();
        }

        found_any = true;
        let pal_name = i18n::t(&char_id);
        let status_translated = i18n::t(&status);
        println!(
            " {:<20} | {:<5} | {:>6}% | {:>6.1} | {}",
            pal_name, level_val, hp_pct, san_val, status_translated
        );
    }

    if !found_any {
        println!(" {}", i18n::t("no_pals_found"));
    }
    println!("\n=================================================================");
}

fn handle_analyzer(
    level_bytes: &[u8],
    player_uid: &str,
    filter_trait: Option<&str>,
    filter_gender: Option<&str>,
    filter_min_level: Option<u32>,
) {
    let characters = scan_character_save_parameters(level_bytes);
    println!("\n=== {} ===\n", i18n::t("analyzer_report_title"));
    println!(
        " {:<20} | {:<5} | {:<6} | {:<5} | {:<5} | {:<5} | {}",
        i18n::t("col_pal"),
        i18n::t("level"),
        i18n::t("gender"),
        "IV HP",
        "IV ATK",
        "IV DEF",
        i18n::t("passives")
    );
    println!("{}", "-".repeat(95));

    let mut found_any = false;
    for char_entry in &characters {
        let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
        if is_player {
            continue;
        }

        let owner_uid =
            extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
        if owner_uid != player_uid {
            continue;
        }

        let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
        if char_id.is_empty() || char_id == "Desconocido" {
            continue;
        }

        let level = extract_byte_prop(&char_entry.raw_data, b"Level\x00");
        let level_val = if level == 0 { 1 } else { level };

        if let Some(min_lvl) = filter_min_level {
            if level_val < min_lvl {
                continue;
            }
        }

        let gender_raw = extract_string_prop(&char_entry.raw_data, b"Gender\x00");
        let gender = if gender_raw.contains("Female") {
            "Female"
        } else if gender_raw.contains("Male") {
            "Male"
        } else {
            "Unknown"
        };

        if let Some(gen) = filter_gender {
            let gen_short = if gen.to_lowercase().starts_with("m") {
                "male"
            } else {
                "female"
            };
            if !gender.to_lowercase().starts_with(gen_short) {
                continue;
            }
        }

        let iv_hp = if crate::scanner::has_prop(&char_entry.raw_data, b"Talent_HP\x00") {
            extract_int_prop(&char_entry.raw_data, b"Talent_HP\x00")
        } else {
            0
        };

        let iv_atk = if crate::scanner::has_prop(&char_entry.raw_data, b"Talent_Shot\x00") {
            extract_int_prop(&char_entry.raw_data, b"Talent_Shot\x00")
        } else {
            0
        };

        let iv_def = if crate::scanner::has_prop(&char_entry.raw_data, b"Talent_Defense\x00") {
            extract_int_prop(&char_entry.raw_data, b"Talent_Defense\x00")
        } else {
            0
        };

        let passive_skills = extract_array_strings(&char_entry.raw_data, b"PassiveSkillList\x00");

        if let Some(tr) = filter_trait {
            let trait_lower = tr.to_lowercase();
            let mut matches_trait = false;
            for p in &passive_skills {
                if p.to_lowercase().contains(&trait_lower)
                    || i18n::t(p).to_lowercase().contains(&trait_lower)
                {
                    matches_trait = true;
                    break;
                }
            }
            if !matches_trait {
                continue;
            }
        }

        found_any = true;
        let pal_name = i18n::t(&char_id);
        let translated_passives: Vec<String> = passive_skills.iter().map(|p| i18n::t(p)).collect();
        println!(
            " {:<20} | {:<5} | {:<6} | {:>5} | {:>5} | {:>5} | {}",
            pal_name,
            level_val,
            gender,
            iv_hp,
            iv_atk,
            iv_def,
            translated_passives.join(", ")
        );
    }

    if !found_any {
        println!(" {}", i18n::t("no_pals_found"));
    }
    println!("\n===============================================================================================");
}

fn handle_breeding(level_bytes: &[u8], player_uid: &str, target_pal: Option<&str>) {
    let characters = scan_character_save_parameters(level_bytes);

    let mut males = std::collections::BTreeSet::new();
    let mut females = std::collections::BTreeSet::new();
    let mut owned_pals = HashSet::new();

    for char_entry in &characters {
        let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
        if is_player {
            continue;
        }

        let owner_uid =
            extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
        if owner_uid == player_uid {
            let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
            if char_id.is_empty() || char_id == "Desconocido" {
                continue;
            }
            let translated_name = i18n::t(&char_id);

            let has_power = BREED_POWER.iter().any(|&(name, _)| name == translated_name);
            if !has_power {
                continue;
            }

            owned_pals.insert(translated_name.clone());

            let gender = extract_string_prop(&char_entry.raw_data, b"Gender\x00");
            if gender.contains("Male") {
                males.insert(translated_name);
            } else if gender.contains("Female") {
                females.insert(translated_name);
            }
        }
    }

    if let Some(target) = target_pal {
        println!(
            "\n=== {} para '{}' ===\n",
            i18n::t("breeding_assistant_title"),
            target
        );
        if let Some(path) = find_breeding_path(&owned_pals, target) {
            if path.is_empty() {
                println!(" Ya posees un {} en tu Palbox.", target);
            } else {
                println!(
                    " Se ha encontrado una ruta de crianza en {} pasos:",
                    path.len()
                );
                for (idx, (p1, p2, child)) in path.iter().enumerate() {
                    println!("   Paso {}: {} + {} -> {}", idx + 1, p1, p2, child);
                }
            }
        } else {
            println!(" No se encontró una ruta de crianza posible para obtener '{}' con tus Pals actuales.", target);
        }
        println!("\n=========================================================");
    } else {
        let mut combinations = Vec::new();
        for male in &males {
            for female in &females {
                let power_a = BREED_POWER
                    .iter()
                    .find(|&&(name, _)| name == *male)
                    .map(|&(_, p)| p)
                    .unwrap_or(1500);
                let power_b = BREED_POWER
                    .iter()
                    .find(|&&(name, _)| name == *female)
                    .map(|&(_, p)| p)
                    .unwrap_or(1500);
                let (child, avg_power) = find_child_pal(power_a, power_b);
                combinations.push((male.clone(), female.clone(), child.to_string(), avg_power));
            }
        }
        combinations.sort_by(|a, b| a.2.cmp(&b.2));

        println!("\n=== {} ===\n", i18n::t("breeding_assistant_title"));
        let males_vec: Vec<String> = males.iter().cloned().collect();
        let females_vec: Vec<String> = females.iter().cloned().collect();

        println!(
            " {}: {}",
            i18n::t("males_available"),
            if males_vec.is_empty() {
                i18n::t("none_m")
            } else {
                males_vec.join(", ")
            }
        );
        println!(
            " {}: {}",
            i18n::t("females_available"),
            if females_vec.is_empty() {
                i18n::t("none_f")
            } else {
                females_vec.join(", ")
            }
        );
        println!("\n {}", i18n::t("possible_combinations_header"));
        println!("--------------------------------------------------");
        for c in &combinations {
            println!(
                " * {} + {} -> {} ({}: {})",
                c.0,
                c.1,
                c.2,
                i18n::t("potencia_promedio"),
                c.3
            );
        }
        println!("\n========================================================");
    }
}

fn handle_clean_seeds(level_bytes: &mut [u8], world_path: &Path) {
    let level_sav = world_path.join("Level.sav");
    let backup_sav = world_path.join("Level.sav.bak");
    println!("Creando copia de seguridad en {}...", backup_sav.display());
    if let Err(e) = std::fs::copy(&level_sav, &backup_sav) {
        println!("Error al crear copia de seguridad: {}", e);
        return;
    }

    let mut header = [0u8; 12];
    {
        let mut file = match std::fs::File::open(&level_sav) {
            Ok(f) => f,
            Err(e) => {
                println!("Error al abrir Level.sav: {}", e);
                return;
            }
        };
        if let Err(e) = file.read_exact(&mut header) {
            println!("Error al leer el encabezado de Level.sav: {}", e);
            return;
        }
    }

    let cleaned = clean_seeds_in_bytes(level_bytes);
    if cleaned.is_empty() {
        println!(" No se encontraron semillas ni objetos basura para limpiar.");
        return;
    }

    println!(" Limpieza completada. Objetos eliminados:");
    for (item_id, count) in &cleaned {
        println!("  - {}: {}", i18n::t(item_id), count);
    }

    println!(" Recomprimiendo y guardando cambios en Level.sav...");
    match compress_and_write_gvas(&level_sav, level_bytes, &header) {
        Ok(_) => println!(" Cambios guardados exitosamente. Por favor, reinicia tu partida si el juego estaba abierto."),
        Err(e) => println!(" Error al guardar cambios en Level.sav: {}", e),
    }
}

fn handle_export(
    level_bytes: &[u8],
    world_path: &Path,
    format: &str,
    file_path: &str,
    player_uid: &str,
) {
    if format.eq_ignore_ascii_case("json") {
        let players = Vec::new();
        let base_camps = scan_base_camps(level_bytes);
        let guilds = scan_guilds(level_bytes);
        let output = OutputJson {
            status: "success".to_string(),
            world_path: world_path.to_string_lossy().into_owned(),
            game_mode: detect_game_mode(world_path),
            players,
            base_camps,
            guilds,
        };
        match serde_json::to_string_pretty(&output) {
            Ok(json_str) => {
                if std::fs::write(file_path, json_str).is_ok() {
                    println!(" Datos exportados exitosamente a JSON en: {}", file_path);
                } else {
                    println!(" Error al escribir el archivo JSON.");
                }
            }
            Err(e) => println!(" Error al serializar a JSON: {}", e),
        }
    } else if format.eq_ignore_ascii_case("csv") {
        let characters = scan_character_save_parameters(level_bytes);
        let mut csv_content = String::new();
        csv_content.push_str("OwnerPlayerUID,Species,Level,Gender,HP,MaxHP,Sanity,Satiety,IV_HP,IV_Atk,IV_Def,Passives\n");

        for char_entry in &characters {
            let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
            if is_player {
                continue;
            }

            let owner_uid =
                extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
            if owner_uid != player_uid {
                continue;
            }

            let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
            if char_id.is_empty() || char_id == "Desconocido" {
                continue;
            }

            let level = extract_byte_prop(&char_entry.raw_data, b"Level\x00");
            let level_val = if level == 0 { 1 } else { level };

            let gender_raw = extract_string_prop(&char_entry.raw_data, b"Gender\x00");
            let gender = if gender_raw.contains("Female") {
                "Female"
            } else if gender_raw.contains("Male") {
                "Male"
            } else {
                "Unknown"
            };

            let hp_cur = extract_fixed_point_prop(&char_entry.raw_data, b"Hp\x00");
            let hp_max = extract_fixed_point_prop(&char_entry.raw_data, b"MaxHp\x00");

            let mut san_val = 100.0;
            if crate::scanner::has_prop(&char_entry.raw_data, b"SanityValue\x00") {
                san_val = extract_float_prop(&char_entry.raw_data, b"SanityValue\x00");
            }

            let mut satiety = 100.0;
            if crate::scanner::has_prop(&char_entry.raw_data, b"FullStomach\x00") {
                satiety = extract_float_prop(&char_entry.raw_data, b"FullStomach\x00");
            }

            let iv_hp = if crate::scanner::has_prop(&char_entry.raw_data, b"Talent_HP\x00") {
                extract_int_prop(&char_entry.raw_data, b"Talent_HP\x00")
            } else {
                0
            };

            let iv_atk = if crate::scanner::has_prop(&char_entry.raw_data, b"Talent_Shot\x00") {
                extract_int_prop(&char_entry.raw_data, b"Talent_Shot\x00")
            } else {
                0
            };

            let iv_def = if crate::scanner::has_prop(&char_entry.raw_data, b"Talent_Defense\x00") {
                extract_int_prop(&char_entry.raw_data, b"Talent_Defense\x00")
            } else {
                0
            };

            let passive_skills =
                extract_array_strings(&char_entry.raw_data, b"PassiveSkillList\x00");
            let passives_str = passive_skills.join(";");

            csv_content.push_str(&format!(
                "{},{},{},{},{:.1},{:.1},{:.1},{:.1},{},{},{},{}\n",
                owner_uid,
                char_id,
                level_val,
                gender,
                hp_cur,
                hp_max,
                san_val,
                satiety,
                iv_hp,
                iv_atk,
                iv_def,
                passives_str
            ));
        }

        if std::fs::write(file_path, csv_content).is_ok() {
            println!(" Datos exportados exitosamente a CSV en: {}", file_path);
        } else {
            println!(" Error al escribir el archivo CSV.");
        }
    } else {
        println!(" Formato no soportado. Usa 'json' o 'csv'.");
    }
}

pub fn find_breeding_path(
    available_pals: &HashSet<String>,
    target: &str,
) -> Option<Vec<(String, String, String)>> {
    if available_pals.contains(target) {
        return Some(vec![]);
    }

    let mut parent_map: HashMap<String, (String, String)> = HashMap::new();
    let mut reached: HashSet<String> = available_pals.clone();
    let mut reached_list: Vec<String> = available_pals.iter().cloned().collect();

    let mut new_additions = true;
    while new_additions {
        new_additions = false;
        let current_reached = reached_list.clone();
        for i in 0..current_reached.len() {
            for j in i..current_reached.len() {
                let parent_a = &current_reached[i];
                let parent_b = &current_reached[j];

                let power_a = BREED_POWER
                    .iter()
                    .find(|&&(name, _)| name == parent_a)
                    .map(|&(_, p)| p);
                let power_b = BREED_POWER
                    .iter()
                    .find(|&&(name, _)| name == parent_b)
                    .map(|&(_, p)| p);

                if let (Some(pa), Some(pb)) = (power_a, power_b) {
                    let (child, _) = find_child_pal(pa, pb);
                    let child_str = child.to_string();

                    if !reached.contains(&child_str) {
                        reached.insert(child_str.clone());
                        reached_list.push(child_str.clone());
                        parent_map.insert(child_str.clone(), (parent_a.clone(), parent_b.clone()));
                        new_additions = true;

                        if child_str == target {
                            return Some(reconstruct_breeding_tree(
                                &parent_map,
                                target,
                                available_pals,
                            ));
                        }
                    }
                }
            }
        }
    }
    None
}

fn reconstruct_breeding_tree(
    parent_map: &HashMap<String, (String, String)>,
    target: &str,
    initial_pals: &HashSet<String>,
) -> Vec<(String, String, String)> {
    let mut steps = Vec::new();
    let mut visited = HashSet::new();

    fn visit(
        node: &str,
        parent_map: &HashMap<String, (String, String)>,
        initial_pals: &HashSet<String>,
        visited: &mut HashSet<String>,
        steps: &mut Vec<(String, String, String)>,
    ) {
        if initial_pals.contains(node) || visited.contains(node) {
            return;
        }
        if let Some((p1, p2)) = parent_map.get(node) {
            visit(p1, parent_map, initial_pals, visited, steps);
            visit(p2, parent_map, initial_pals, visited, steps);
            steps.push((p1.clone(), p2.clone(), node.to_string()));
            visited.insert(node.to_string());
        }
    }

    visit(target, parent_map, initial_pals, &mut visited, &mut steps);
    steps
}
