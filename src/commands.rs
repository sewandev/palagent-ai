use crate::decompress::decompress_gvas;
use crate::i18n;
use crate::models::{OutputJson, PalSummary, PlayerSummary};
use crate::scanner::{
    clean_seeds_in_bytes, compress_and_write_gvas, extract_array_strings, extract_bool_prop,
    extract_byte_prop, extract_enum_prop, extract_fixed_point_prop, extract_float_prop,
    extract_guid_bytes_prop, extract_guid_prop, extract_int64_prop, extract_int_prop,
    extract_string_prop, find_chest_containers, format_guid, parse_container_items,
    scan_base_camps, scan_character_save_parameters, scan_guilds,
};
use crate::utils::{detect_game_mode, find_child_pal};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn run_time_command(world_path: &Path, is_json: bool) {
    let level_sav = world_path.join("Level.sav");
    let bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress Level.sav: {}", e) })
                );
            } else {
                println!("Failed to decompress Level.sav: {}", e);
            }
            return;
        }
    };
    let ticks = extract_int64_prop(&bytes, b"GameDateTimeTicks\x00");
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

    if is_json {
        let out_json = serde_json::json!({
            "status": "success",
            "total_ticks": ticks,
            "game_day": day_number,
            "in_game_time": format!("{:02}:{:02}", hour, minute),
            "is_day": is_day
        });
        println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
    } else {
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
}

pub fn run_settings_command(world_path: &Path, is_json: bool) {
    let opt_sav = world_path.join("WorldOption.sav");
    if !opt_sav.exists() {
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": format!("WorldOption.sav not found at {}.", opt_sav.display()) })
            );
        } else {
            println!("WorldOption.sav not found at {}.", opt_sav.display());
        }
        return;
    }
    let bytes = match decompress_gvas(&opt_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress WorldOption.sav: {}", e) })
                );
            } else {
                println!("Failed to decompress WorldOption.sav: {}", e);
            }
            return;
        }
    };

    let difficulty = extract_enum_prop(&bytes, b"Difficulty\x00");
    let exp_rate = extract_float_prop(&bytes, b"ExpRate\x00");
    let capture_rate = extract_float_prop(&bytes, b"PalCaptureRate\x00");
    let spawn_rate = extract_float_prop(&bytes, b"PalSpawnNumRate\x00");
    let pal_atk = extract_float_prop(&bytes, b"PalDamageRateAttack\x00");
    let pal_def = extract_float_prop(&bytes, b"PalDamageRateDefense\x00");
    let player_atk = extract_float_prop(&bytes, b"PlayerDamageRateAttack\x00");
    let player_def = extract_float_prop(&bytes, b"PlayerDamageRateDefense\x00");
    let player_stamina = extract_float_prop(&bytes, b"PlayerStaminaDecreaceRate\x00");
    let player_stomach = extract_float_prop(&bytes, b"PlayerStomachDecreaceRate\x00");
    let player_regen = extract_float_prop(&bytes, b"PlayerAutoHPRegeneRate\x00");
    let egg_time = extract_float_prop(&bytes, b"PalEggDefaultHatchingTime\x00");
    let death_penalty = extract_enum_prop(&bytes, b"DeathPenalty\x00");
    let is_multi = extract_bool_prop(&bytes, b"bIsMultiplay\x00");
    let is_pvp = extract_bool_prop(&bytes, b"bIsPvP\x00");
    let fast_travel = extract_bool_prop(&bytes, b"bEnableFastTravel\x00");
    let build_limit = extract_bool_prop(&bytes, b"bBuildAreaLimit\x00");
    let max_guild = extract_int_prop(&bytes, b"GuildPlayerMaxNum\x00");
    let max_base = extract_int_prop(&bytes, b"BaseCampMaxNumInGuild\x00");

    if is_json {
        let out_json = serde_json::json!({
            "status": "success",
            "settings": {
                "difficulty": difficulty,
                "exp_rate": exp_rate,
                "pal_capture_rate": capture_rate,
                "pal_spawn_num_rate": spawn_rate,
                "pal_damage_rate_attack": pal_atk,
                "pal_damage_rate_defense": pal_def,
                "player_damage_rate_attack": player_atk,
                "player_damage_rate_defense": player_def,
                "player_stamina_decrease_rate": player_stamina,
                "player_stomach_decrease_rate": player_stomach,
                "player_auto_hp_regene_rate": player_regen,
                "pal_egg_default_hatching_time": egg_time,
                "death_penalty": death_penalty,
                "is_multiplay": is_multi,
                "is_pvp": is_pvp,
                "enable_fast_travel": fast_travel,
                "build_area_limit": build_limit,
                "guild_player_max_num": max_guild,
                "base_camp_max_num_in_guild": max_base
            }
        });
        println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
    } else {
        println!("==================================================");
        println!("   {}", i18n::t("settings_report_title"));
        println!("==================================================");
        println!(" {:<40} : {}", i18n::t("Difficulty"), difficulty);
        println!(" {:<40} : {:.2}", i18n::t("ExpRate"), exp_rate);
        println!(" {:<40} : {:.2}", i18n::t("PalCaptureRate"), capture_rate);
        println!(" {:<40} : {:.2}", i18n::t("PalSpawnNumRate"), spawn_rate);
        println!(" {:<40} : {:.2}", i18n::t("PalDamageRateAttack"), pal_atk);
        println!(" {:<40} : {:.2}", i18n::t("PalDamageRateDefense"), pal_def);
        println!(
            " {:<40} : {:.2}",
            i18n::t("PlayerDamageRateAttack"),
            player_atk
        );
        println!(
            " {:<40} : {:.2}",
            i18n::t("PlayerDamageRateDefense"),
            player_def
        );
        println!(
            " {:<40} : {:.2}",
            i18n::t("PlayerStaminaDecreaceRate"),
            player_stamina
        );
        println!(
            " {:<40} : {:.2}",
            i18n::t("PlayerStomachDecreaceRate"),
            player_stomach
        );
        println!(
            " {:<40} : {:.2}",
            i18n::t("PlayerAutoHPRegeneRate"),
            player_regen
        );
        println!(
            " {:<40} : {:.2}",
            i18n::t("PalEggDefaultHatchingTime"),
            egg_time
        );
        println!(
            " {:<40} : {}",
            i18n::t("DeathPenalty"),
            i18n::t(&death_penalty)
        );
        println!(
            " {:<40} : {}",
            i18n::t("bIsMultiplay"),
            if is_multi {
                i18n::t("Si")
            } else {
                i18n::t("No")
            }
        );
        println!(
            " {:<40} : {}",
            i18n::t("bIsPvP"),
            if is_pvp { i18n::t("Si") } else { i18n::t("No") }
        );
        println!(
            " {:<40} : {}",
            i18n::t("bEnableFastTravel"),
            if fast_travel {
                i18n::t("Si")
            } else {
                i18n::t("No")
            }
        );
        println!(
            " {:<40} : {}",
            i18n::t("bBuildAreaLimit"),
            if build_limit {
                i18n::t("Si")
            } else {
                i18n::t("No")
            }
        );
        println!(" {:<40} : {}", i18n::t("GuildPlayerMaxNum"), max_guild);
        println!(" {:<40} : {}", i18n::t("BaseCampMaxNumInGuild"), max_base);
        println!("==================================================");
    }
}

pub fn run_search_chest_command(world_path: &Path, query: &str, is_json: bool) {
    let level_sav = world_path.join("Level.sav");
    let bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress Level.sav: {}", e) })
                );
            } else {
                println!("Failed to decompress Level.sav: {}", e);
            }
            return;
        }
    };

    let query_lower = query.to_lowercase();
    let chests = find_chest_containers(&bytes);

    if is_json {
        let mut results = Vec::new();
        for (guid, chest_type, coords) in &chests {
            let items = parse_container_items(&bytes, guid);
            for item in items {
                let item_name = i18n::t(&item.item_id);
                if item.item_id.to_lowercase().contains(&query_lower)
                    || item_name.to_lowercase().contains(&query_lower)
                {
                    let guid_str = format_guid(guid);
                    results.push(serde_json::json!({
                        "item_id": item.item_id,
                        "item_name": item_name,
                        "count": item.count,
                        "container_guid": guid_str,
                        "chest_type": chest_type,
                        "coordinates": {
                            "x": coords.0,
                            "y": coords.1,
                            "z": coords.2
                        }
                    }));
                }
            }
        }
        let out_json = serde_json::json!({
            "status": "success",
            "query": query,
            "results": results
        });
        println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
    } else {
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
            let items = parse_container_items(&bytes, &guid);
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
        println!("\n=====================================================================================");
    }
}

pub fn run_breeding_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
    let level_sav = world_path.join("Level.sav");
    let level_bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress Level.sav: {}", e) })
                );
            } else {
                println!("Failed to decompress Level.sav: {}", e);
            }
            return;
        }
    };

    let characters = scan_character_save_parameters(&level_bytes);

    let mut player_uid = target_uid
        .unwrap_or("00000000-0000-0000-0000-000000000001")
        .to_string();
    if target_uid.is_none() {
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
                            player_uid = guid_str;
                            break;
                        }
                    }
                }
            }
        }
    }

    let mut males = std::collections::BTreeSet::new();
    let mut females = std::collections::BTreeSet::new();

    for char_entry in &characters {
        let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
        if is_player {
            continue;
        }

        let owner_uid =
            extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
        if owner_uid == player_uid {
            let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
            if char_id.is_empty() || !crate::db::is_valid_pal(&char_id) {
                continue;
            }
            let translated_name = i18n::t(&char_id);

            let gender = extract_string_prop(&char_entry.raw_data, b"Gender\x00");
            if gender.contains("Male") {
                males.insert(translated_name);
            } else if gender.contains("Female") {
                females.insert(translated_name);
            }
        }
    }

    let mut combinations = Vec::new();
    for male in &males {
        for female in &females {
            let (child, avg_power) = find_child_pal(male, female);
            combinations.push((male.clone(), female.clone(), child, avg_power));
        }
    }
    combinations.sort_by(|a, b| a.2.cmp(&b.2));

    if is_json {
        let out_json = serde_json::json!({
            "status": "success",
            "males_available": males,
            "females_available": females,
            "combinations": combinations.iter().map(|c| {
                serde_json::json!({
                    "father": c.0,
                    "mother": c.1,
                    "child": c.2,
                    "child_translated": i18n::t(&c.2),
                    "average_power": c.3
                })
            }).collect::<Vec<serde_json::Value>>()
        });
        println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
    } else {
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
        println!();

        if males.is_empty() || females.is_empty() {
            println!(" {}", i18n::t("breeding_needs_both"));
            return;
        }

        println!(
            " {:<15} + {:<15} => {:<15} ({})",
            i18n::t("col_father"),
            i18n::t("col_mother"),
            i18n::t("col_child"),
            i18n::t("col_avg_power")
        );
        println!("{}", "-".repeat(75));
        for (male, female, child, avg_power) in combinations {
            println!(
                " {:<15} + {:<15} => {:<15} (Poder: {})",
                male,
                female,
                i18n::t(&child),
                avg_power
            );
        }
        println!("\n==========================================================================");
    }
}

pub fn run_progress_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
    let mut player_sav = None;
    let players_dir = world_path.join("Players");
    if let Some(uid) = target_uid {
        let stem = uid.replace("-", "");
        let path = players_dir.join(format!("{}.sav", stem));
        if path.exists() {
            player_sav = Some(path);
        }
    }
    if player_sav.is_none() {
        if let Ok(entries) = std::fs::read_dir(&players_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sav") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem != "00000000000000000000000000000001" && stem.len() == 32 {
                            player_sav = Some(path);
                            break;
                        }
                    }
                }
            }
        }
    }

    let player_sav = match player_sav {
        Some(p) => p,
        None => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": "Player save file not found." })
                );
            } else {
                println!("Player save file not found.");
            }
            return;
        }
    };

    let bytes = match decompress_gvas(&player_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress player save: {}", e) })
                );
            } else {
                println!("Failed to decompress player save: {}", e);
            }
            return;
        }
    };

    let relics = crate::scanner::extract_map_keys(&bytes, b"RelicObtainForInstanceFlag\x00");
    let notes = crate::scanner::extract_map_keys(&bytes, b"NoteObtainForInstanceFlag\x00");
    let fast_travels = crate::scanner::extract_map_keys(&bytes, b"FastTravelPointUnlockFlag\x00");
    let areas = crate::scanner::extract_map_keys(&bytes, b"FindAreaFlagMap\x00");
    let captures = crate::scanner::extract_map_counts(&bytes, b"PalCaptureCount\x00");

    if is_json {
        let captures_list: Vec<serde_json::Value> = captures
            .into_iter()
            .map(|(pal_id, count)| {
                serde_json::json!({
                    "pal_id": pal_id,
                    "pal_name": i18n::t(&pal_id),
                    "captured_count": count,
                    "bonus_completed": count >= 10,
                    "missing_for_bonus": 10_u32.saturating_sub(count)
                })
            })
            .collect();

        let out_json = serde_json::json!({
            "status": "success",
            "progress": {
                "relics_found": relics.len(),
                "relics_total": 435,
                "notes_found": notes.len(),
                "notes_total": 45,
                "fast_travels_unlocked": fast_travels.len(),
                "fast_travels_total": 57,
                "areas_discovered": areas.len(),
                "captures": captures_list
            }
        });
        println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
    } else {
        println!("\n=== {} ===\n", i18n::t("progress_report_title"));
        println!(
            " {:<35} : {}/435 ({:.2}%)",
            i18n::t("relics_progress"),
            relics.len(),
            (relics.len() as f64 / 435.0) * 100.0
        );
        println!(
            " {:<35} : {}/45 ({:.2}%)",
            i18n::t("notes_progress"),
            notes.len(),
            (notes.len() as f64 / 45.0) * 100.0
        );
        println!(
            " {:<35} : {}/57 ({:.2}%)",
            i18n::t("fast_travels_progress"),
            fast_travels.len(),
            (fast_travels.len() as f64 / 57.0) * 100.0
        );
        println!(" {:<35} : {}", i18n::t("areas_discovered"), areas.len());
        println!("\n=======================================================");

        println!("\n=== {} ===\n", i18n::t("capture_bonus_title"));
        println!(
            " {:<20} | {:<10} | {}",
            i18n::t("col_pal"),
            i18n::t("col_captured"),
            i18n::t("col_bonus_status")
        );
        println!("{}", "-".repeat(55));

        if captures.is_empty() {
            println!(" {}", i18n::t("no_captures_yet"));
        } else {
            let mut captures_sorted: Vec<(String, u32)> = captures.into_iter().collect();
            captures_sorted.sort_by_key(|a| i18n::t(&a.0));

            for (pal_id, count) in captures_sorted {
                let translated_name = i18n::t(&pal_id);
                let status = if count >= 10 {
                    i18n::t("bonus_completed")
                } else {
                    format!("{} {}", i18n::t("bonus_missing"), 10 - count)
                };
                println!(" {:<20} | {:<10} | {}", translated_name, count, status);
            }
        }
        println!("\n=======================================================");
    }
}

pub fn run_clean_seeds_command(world_path: &Path, is_json: bool) {
    let level_sav = world_path.join("Level.sav");
    let backup_sav = world_path.join("Level.sav.bak");

    if !is_json {
        println!("Creating backup at {}...", backup_sav.display());
    }
    if let Err(e) = std::fs::copy(&level_sav, &backup_sav) {
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": format!("Failed to create backup: {}", e) })
            );
        } else {
            println!("Failed to create backup: {}", e);
        }
        return;
    }

    let mut orig_file = match File::open(&level_sav) {
        Ok(f) => f,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to open Level.sav: {}", e) })
                );
            } else {
                println!("Failed to open Level.sav: {}", e);
            }
            return;
        }
    };
    let mut header = [0u8; 12];
    if let Err(e) = orig_file.read_exact(&mut header) {
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": format!("Failed to read Level.sav header: {}", e) })
            );
        } else {
            println!("Failed to read Level.sav header: {}", e);
        }
        return;
    }
    drop(orig_file);

    if !is_json {
        println!("Decompressing Level.sav...");
    }
    let mut bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress Level.sav: {}", e) })
                );
            } else {
                println!("Failed to decompress Level.sav: {}", e);
            }
            return;
        }
    };

    if !is_json {
        println!("Searching and cleaning seed items in-place...");
    }
    let cleaned = clean_seeds_in_bytes(&mut bytes);

    if cleaned.is_empty() {
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "success", "cleaned_count": 0, "cleaned_items": [] })
            );
        } else {
            println!("No seed items found in containers.");
        }
        return;
    }

    if !is_json {
        println!("Removed items:");
        for (item, count) in &cleaned {
            println!(" - {} (x{})", i18n::t(item), count);
        }
    }

    if !is_json {
        println!("Re-compressing and writing back Level.sav...");
    }
    match compress_and_write_gvas(&level_sav, &bytes, &header) {
        Ok(_) => {
            if is_json {
                let cleaned_list: Vec<serde_json::Value> = cleaned.iter().map(|(item, count)| {
                    serde_json::json!({ "item_id": item, "item_name": i18n::t(item), "count": count })
                }).collect();
                let out_json = serde_json::json!({
                    "status": "success",
                    "cleaned_count": cleaned.len(),
                    "cleaned_items": cleaned_list
                });
                println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
            } else {
                println!("Inventory cleanup completed successfully!");
            }
        }
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to save Level.sav: {}", e) })
                );
            } else {
                println!("Failed to save Level.sav: {}", e);
            }
        }
    }
}

pub fn run_monitor_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
    let level_sav = world_path.join("Level.sav");
    let level_bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress Level.sav: {}", e) })
                );
            } else {
                println!("Failed to decompress Level.sav: {}", e);
            }
            return;
        }
    };

    let characters = scan_character_save_parameters(&level_bytes);

    if is_json {
        let mut pals_list = Vec::new();
        for char_entry in &characters {
            let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
            if is_player {
                continue;
            }

            let owner_uid =
                extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
            if let Some(uid) = target_uid {
                if owner_uid != uid {
                    continue;
                }
            }

            let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
            if char_id.is_empty() || char_id == "Desconocido" {
                continue;
            }

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

            if hp_pct < 30 {
                status = "status_gravely_injured".to_string();
            }

            pals_list.push(serde_json::json!({
                "pal_id": char_id,
                "pal_name": i18n::t(&char_id),
                "hp_current": hp_cur,
                "hp_max": hp_max,
                "hp_percent": hp_pct,
                "sanity": san_val,
                "satiety": satiety,
                "status_key": status,
                "status_translated": i18n::t(&status)
            }));
        }
        let out_json = serde_json::json!({
            "status": "success",
            "pals": pals_list
        });
        println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
    } else {
        println!("\n=== {} ===\n", i18n::t("monitor_report_title"));
        println!(
            " {:<15} | {:<12} | {:<15} | {:<20} | {}",
            i18n::t("col_pal"),
            i18n::t("health_hp"),
            i18n::t("col_san"),
            i18n::t("col_hunger"),
            i18n::t("col_status")
        );
        println!("{}", "-".repeat(80));

        let mut count = 0;
        for char_entry in &characters {
            let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
            if is_player {
                continue;
            }

            let owner_uid =
                extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
            if let Some(uid) = target_uid {
                if owner_uid != uid {
                    continue;
                }
            }

            let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
            if char_id.is_empty() || char_id == "Desconocido" {
                continue;
            }

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

            let pal_name = i18n::t(&char_id);

            let mut status = i18n::t("status_excellent");
            if san_val < 50.0 {
                status = i18n::t("status_very_stressed");
            } else if san_val < 70.0 {
                status = i18n::t("status_stressed");
            }

            if satiety < 40.0 {
                status = i18n::t("status_critically_hungry");
            } else if satiety < 70.0 {
                status = i18n::t("status_needs_food");
            }

            if hp_pct < 30 {
                status = i18n::t("status_gravely_injured");
            }

            println!(
                " {:<15} | {:>10}% | {:>12.1}/100 | {:>17.1}/100 | {}",
                pal_name, hp_pct, san_val, satiety, status
            );
            count += 1;
        }

        if count == 0 {
            println!(" No Pals found in Level.sav.");
        }

        println!("\n{}", i18n::t("monitor_note"));
        println!(
            "================================================================================"
        );
    }
}

pub fn run_analyzer_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
    let level_sav = world_path.join("Level.sav");
    let level_bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Failed to decompress Level.sav: {}", e) })
                );
            } else {
                println!("Failed to decompress Level.sav: {}", e);
            }
            return;
        }
    };

    let characters = scan_character_save_parameters(&level_bytes);

    if is_json {
        let mut pals_list = Vec::new();
        for char_entry in &characters {
            let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
            if is_player {
                continue;
            }

            let owner_uid =
                extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
            if let Some(uid) = target_uid {
                if owner_uid != uid {
                    continue;
                }
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
            let passive_skills_translated: Vec<String> =
                passive_skills.iter().map(|p| i18n::t(p)).collect();

            pals_list.push(serde_json::json!({
                "pal_id": char_id,
                "pal_name": i18n::t(&char_id),
                "level": level_val,
                "gender": gender,
                "gender_translated": i18n::t(gender),
                "ivs": {
                    "hp": iv_hp,
                    "attack": iv_atk,
                    "defense": iv_def
                },
                "passive_skills": passive_skills,
                "passive_skills_translated": passive_skills_translated,
                "work_suitabilities": crate::db::get_pal_suitabilities(&char_id)
            }));
        }
        let out_json = serde_json::json!({
            "status": "success",
            "pals": pals_list
        });
        println!("{}", serde_json::to_string_pretty(&out_json).unwrap());
    } else {
        println!("\n=== {} ===\n", i18n::t("analyzer_report_title"));
        println!(
            " {:<15} | {:<3} | {:<6} | {:<8} | {:<9} | {:<10} | {}",
            i18n::t("col_pal"),
            i18n::t("col_level_short"),
            i18n::t("col_gender"),
            i18n::t("col_iv_hp"),
            i18n::t("col_iv_atk"),
            i18n::t("col_iv_def"),
            i18n::t("col_passives")
        );
        println!("{}", "-".repeat(100));

        let mut count = 0;
        for char_entry in &characters {
            let is_player = extract_bool_prop(&char_entry.raw_data, b"IsPlayer\x00");
            if is_player {
                continue;
            }

            let owner_uid =
                extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
            if let Some(uid) = target_uid {
                if owner_uid != uid {
                    continue;
                }
            }

            let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
            if char_id.is_empty() || char_id == "Desconocido" {
                continue;
            }

            let level = extract_byte_prop(&char_entry.raw_data, b"Level\x00");
            let level_val = if level == 0 { 1 } else { level };

            let gender_raw = extract_string_prop(&char_entry.raw_data, b"Gender\x00");
            let gender = if gender_raw.contains("Female") {
                i18n::t("Female")
            } else if gender_raw.contains("Male") {
                i18n::t("Male")
            } else {
                "S/N".to_string()
            };

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
            let passives_str = if passive_skills.is_empty() {
                i18n::t("no_passives")
            } else {
                passive_skills
                    .iter()
                    .map(|p| i18n::t(p))
                    .collect::<Vec<String>>()
                    .join(", ")
            };

            let pal_name = i18n::t(&char_id);

            println!(
                " {:<15} | {:<3} | {:<6} | {:<8} | {:<9} | {:<10} | {}",
                pal_name, level_val, gender, iv_hp, iv_atk, iv_def, passives_str
            );
            count += 1;
        }

        if count == 0 {
            println!(" No Pals found in Level.sav.");
        }

        println!("\n{}", i18n::t("analyzer_note"));
        println!("==========================================================================================");
    }
}

pub fn run_full_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
    let level_sav = world_path.join("Level.sav");
    if !level_sav.exists() {
        let err_json = serde_json::json!({
            "status": "error",
            "message": format!("Level.sav not found at {}", level_sav.display())
        });
        if is_json {
            println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
        } else {
            println!("Level.sav not found at {}", level_sav.display());
        }
        return;
    }

    let level_bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            let err_json = serde_json::json!({
                "status": "error",
                "message": format!("Failed to decompress Level.sav: {}", e)
            });
            if is_json {
                println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
            } else {
                println!("Failed to decompress Level.sav: {}", e);
            }
            return;
        }
    };

    let characters = scan_character_save_parameters(&level_bytes);

    let players_dir = world_path.join("Players");
    let mut players = Vec::new();

    if players_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&players_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sav") {
                    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let player_uid_str = if file_stem.len() == 32 {
                        format!(
                            "{}-{}-{}-{}-{}",
                            &file_stem[0..8],
                            &file_stem[8..12],
                            &file_stem[12..16],
                            &file_stem[16..20],
                            &file_stem[20..32]
                        )
                    } else {
                        file_stem.to_string()
                    };

                    if let Some(uid) = target_uid {
                        if player_uid_str != uid {
                            continue;
                        }
                    }

                    if let Ok(player_bytes) = decompress_gvas(&path) {
                        let player_char_entry =
                            characters.iter().find(|c| c.player_uid == player_uid_str);
                        let mut level = 1;
                        let mut exp = 0;
                        let mut hp = 0.0;
                        let mut max_hp = 0.0;
                        let mut full_stomach = 0.0;
                        let mut physical_health =
                            "EPalStatusPhysicalHealthType::Normal".to_string();
                        let mut nickname = if file_stem == "00000000000000000000000000000001" {
                            "Host Player".to_string()
                        } else {
                            format!("Player_{}", &file_stem[0..8])
                        };

                        if let Some(char_entry) = player_char_entry {
                            nickname = extract_string_prop(&char_entry.raw_data, b"NickName\x00");
                            if nickname.is_empty() {
                                nickname = if file_stem == "00000000000000000000000000000001" {
                                    "Host Player".to_string()
                                } else {
                                    format!("Player_{}", &file_stem[0..8])
                                };
                            }
                            level = extract_byte_prop(&char_entry.raw_data, b"Level\x00");
                            if level == 0 {
                                level = 1;
                            }
                            exp = extract_int64_prop(&char_entry.raw_data, b"Exp\x00");
                            hp = extract_fixed_point_prop(&char_entry.raw_data, b"Hp\x00");
                            max_hp = extract_fixed_point_prop(&char_entry.raw_data, b"MaxHp\x00");
                            full_stomach =
                                extract_float_prop(&char_entry.raw_data, b"FullStomach\x00");
                            let ph =
                                extract_string_prop(&char_entry.raw_data, b"PhysicalHealth\x00");
                            if !ph.is_empty() {
                                physical_health = ph;
                            }
                        }

                        let otomo_bytes = extract_guid_bytes_prop(
                            &player_bytes,
                            b"OtomoCharacterContainerId\x00",
                        );
                        let common_bytes =
                            extract_guid_bytes_prop(&player_bytes, b"CommonContainerId\x00");
                        let weapon_bytes =
                            extract_guid_bytes_prop(&player_bytes, b"WeaponLoadOutContainerId\x00");
                        let armor_bytes = extract_guid_bytes_prop(
                            &player_bytes,
                            b"PlayerEquipArmorContainerId\x00",
                        );
                        let otomo_id = otomo_bytes.map(|b| format_guid(&b)).unwrap_or_default();

                        let mut common_inventory = Vec::new();
                        if let Some(guid) = common_bytes {
                            common_inventory = parse_container_items(&level_bytes, &guid);
                        }

                        let mut weapons = Vec::new();
                        if let Some(guid) = weapon_bytes {
                            weapons = parse_container_items(&level_bytes, &guid);
                        }

                        let mut armor = Vec::new();
                        if let Some(guid) = armor_bytes {
                            armor = parse_container_items(&level_bytes, &guid);
                        }

                        let palbox_bytes =
                            extract_guid_bytes_prop(&player_bytes, b"PalStorageContainerId\x00");
                        let palbox_id = palbox_bytes.map(|b| format_guid(&b)).unwrap_or_default();

                        let mut active_pals = Vec::new();
                        let mut palbox_pals = Vec::new();
                        let player_instance_id = player_char_entry
                            .map(|c| c.instance_id.clone())
                            .unwrap_or_default();
                        for char_entry in &characters {
                            if !player_instance_id.is_empty()
                                && char_entry.instance_id != player_instance_id
                            {
                                let owner_uid =
                                    extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00")
                                        .unwrap_or_default();
                                if owner_uid == player_uid_str {
                                    let pal_container_id =
                                        extract_guid_prop(&char_entry.raw_data, b"ContainerId\x00")
                                            .unwrap_or_default();
                                    if !otomo_id.is_empty() && pal_container_id == otomo_id {
                                        let char_id = extract_string_prop(
                                            &char_entry.raw_data,
                                            b"CharacterID\x00",
                                        );
                                        let pal_gender = extract_string_prop(
                                            &char_entry.raw_data,
                                            b"Gender\x00",
                                        );
                                        let pal_level =
                                            extract_byte_prop(&char_entry.raw_data, b"Level\x00");
                                        let pal_exp =
                                            extract_int64_prop(&char_entry.raw_data, b"Exp\x00");
                                        let pal_hp = extract_fixed_point_prop(
                                            &char_entry.raw_data,
                                            b"Hp\x00",
                                        );
                                        let pal_max_hp = extract_fixed_point_prop(
                                            &char_entry.raw_data,
                                            b"MaxHp\x00",
                                        );
                                        let pal_satiety = extract_float_prop(
                                            &char_entry.raw_data,
                                            b"FullStomach\x00",
                                        );
                                        let pal_ph = extract_string_prop(
                                            &char_entry.raw_data,
                                            b"PhysicalHealth\x00",
                                        );
                                        let friendship = extract_int_prop(
                                            &char_entry.raw_data,
                                            b"FriendshipPoint\x00",
                                        )
                                            as u32;
                                        let slot_index = extract_int_prop(
                                            &char_entry.raw_data,
                                            b"SlotIndex\x00",
                                        )
                                            as u32;

                                        let mut talents = HashMap::new();
                                        if crate::scanner::has_prop(
                                            &char_entry.raw_data,
                                            b"Talent_HP\x00",
                                        ) {
                                            talents.insert(
                                                "HP".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_HP\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if crate::scanner::has_prop(
                                            &char_entry.raw_data,
                                            b"Talent_Shot\x00",
                                        ) {
                                            talents.insert(
                                                "Shot".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_Shot\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if crate::scanner::has_prop(
                                            &char_entry.raw_data,
                                            b"Talent_Defense\x00",
                                        ) {
                                            talents.insert(
                                                "Defense".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_Defense\x00",
                                                )
                                                    as u32,
                                            );
                                        }

                                        let passive_skills = extract_array_strings(
                                            &char_entry.raw_data,
                                            b"PassiveSkillList\x00",
                                        );

                                        active_pals.push(PalSummary {
                                            character_id: if char_id.is_empty() {
                                                "Unknown".to_string()
                                            } else {
                                                char_id
                                            },
                                            gender: if pal_gender.is_empty() {
                                                "EPalGenderType::Male".to_string()
                                            } else {
                                                pal_gender
                                            },
                                            level: if pal_level == 0 { 1 } else { pal_level },
                                            exp: pal_exp,
                                            hp: pal_hp,
                                            max_hp: pal_max_hp,
                                            satiety: pal_satiety,
                                            physical_health: if pal_ph.is_empty() {
                                                "EPalStatusPhysicalHealthType::Normal".to_string()
                                            } else {
                                                pal_ph
                                            },
                                            friendship,
                                            talents,
                                            passive_skills,
                                            slot_index,
                                        });
                                    } else if !palbox_id.is_empty() && pal_container_id == palbox_id
                                    {
                                        let char_id = extract_string_prop(
                                            &char_entry.raw_data,
                                            b"CharacterID\x00",
                                        );
                                        let pal_gender = extract_string_prop(
                                            &char_entry.raw_data,
                                            b"Gender\x00",
                                        );
                                        let pal_level =
                                            extract_byte_prop(&char_entry.raw_data, b"Level\x00");
                                        let pal_exp =
                                            extract_int64_prop(&char_entry.raw_data, b"Exp\x00");
                                        let pal_hp = extract_fixed_point_prop(
                                            &char_entry.raw_data,
                                            b"Hp\x00",
                                        );
                                        let pal_max_hp = extract_fixed_point_prop(
                                            &char_entry.raw_data,
                                            b"MaxHp\x00",
                                        );
                                        let pal_satiety = extract_float_prop(
                                            &char_entry.raw_data,
                                            b"FullStomach\x00",
                                        );
                                        let pal_ph = extract_string_prop(
                                            &char_entry.raw_data,
                                            b"PhysicalHealth\x00",
                                        );
                                        let friendship = extract_int_prop(
                                            &char_entry.raw_data,
                                            b"FriendshipPoint\x00",
                                        )
                                            as u32;
                                        let slot_index = extract_int_prop(
                                            &char_entry.raw_data,
                                            b"SlotIndex\x00",
                                        )
                                            as u32;

                                        let mut talents = HashMap::new();
                                        if crate::scanner::has_prop(
                                            &char_entry.raw_data,
                                            b"Talent_HP\x00",
                                        ) {
                                            talents.insert(
                                                "HP".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_HP\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if crate::scanner::has_prop(
                                            &char_entry.raw_data,
                                            b"Talent_Shot\x00",
                                        ) {
                                            talents.insert(
                                                "Shot".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_Shot\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if crate::scanner::has_prop(
                                            &char_entry.raw_data,
                                            b"Talent_Defense\x00",
                                        ) {
                                            talents.insert(
                                                "Defense".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_Defense\x00",
                                                )
                                                    as u32,
                                            );
                                        }

                                        let passive_skills = extract_array_strings(
                                            &char_entry.raw_data,
                                            b"PassiveSkillList\x00",
                                        );

                                        palbox_pals.push(PalSummary {
                                            character_id: if char_id.is_empty() {
                                                "Unknown".to_string()
                                            } else {
                                                char_id
                                            },
                                            gender: if pal_gender.is_empty() {
                                                "EPalGenderType::Male".to_string()
                                            } else {
                                                pal_gender
                                            },
                                            level: if pal_level == 0 { 1 } else { pal_level },
                                            exp: pal_exp,
                                            hp: pal_hp,
                                            max_hp: pal_max_hp,
                                            satiety: pal_satiety,
                                            physical_health: if pal_ph.is_empty() {
                                                "EPalStatusPhysicalHealthType::Normal".to_string()
                                            } else {
                                                pal_ph
                                            },
                                            friendship,
                                            talents,
                                            passive_skills,
                                            slot_index,
                                        });
                                    }
                                }
                            }
                        }

                        active_pals.sort_by_key(|p| p.slot_index);
                        palbox_pals.sort_by_key(|p| p.slot_index);

                        let technology_points =
                            extract_int_prop(&player_bytes, b"TechnologyPoint\x00") as u32;
                        let unlocked_technologies = extract_array_strings(
                            &player_bytes,
                            b"UnlockedRecipeTechnologyNames\x00",
                        );
                        let active_quest = extract_string_prop(&player_bytes, b"QuestName\x00");
                        let completed_quests =
                            extract_array_strings(&player_bytes, b"CompletedQuestArray\x00");

                        let relics_found =
                            extract_int_prop(&player_bytes, b"RelicPossessNum\x00") as u32;
                        let fast_travel_points = crate::scanner::extract_map_keys(
                            &player_bytes,
                            b"FastTravelPointUnlockFlag\x00",
                        );
                        let notes_found = crate::scanner::extract_map_keys(
                            &player_bytes,
                            b"NoteObtainForInstanceFlag\x00",
                        );
                        let npc_talk_counts = crate::scanner::extract_map_counts(
                            &player_bytes,
                            b"NPCTalkCountMap\x00",
                        );

                        let mut customization = HashMap::new();
                        let b_mesh = extract_string_prop(&player_bytes, b"BodyMeshName\x00");
                        if !b_mesh.is_empty() {
                            customization.insert("BodyMeshName".to_string(), Value::String(b_mesh));
                        }
                        let h_mesh = extract_string_prop(&player_bytes, b"HeadMeshName\x00");
                        if !h_mesh.is_empty() {
                            customization.insert("HeadMeshName".to_string(), Value::String(h_mesh));
                        }
                        let hair_mesh = extract_string_prop(&player_bytes, b"HairMeshName\x00");
                        if !hair_mesh.is_empty() {
                            customization
                                .insert("HairMeshName".to_string(), Value::String(hair_mesh));
                        }
                        let voice_id = extract_int_prop(&player_bytes, b"VoiceID\x00");
                        if voice_id > 0 {
                            customization
                                .insert("VoiceID".to_string(), Value::Number(voice_id.into()));
                        }

                        players.push(PlayerSummary {
                            player_uid: player_uid_str,
                            instance_id: player_char_entry
                                .map(|c| c.instance_id.clone())
                                .unwrap_or_default(),
                            nickname,
                            level,
                            exp,
                            hp,
                            max_hp,
                            full_stomach,
                            physical_health,
                            technology_points,
                            customization,
                            unlocked_technologies,
                            active_quest,
                            completed_quests,
                            fast_travel_points,
                            relics_found,
                            notes_found,
                            npc_talk_counts,
                            common_inventory,
                            weapons,
                            armor,
                            active_pals,
                            palbox_pals,
                        });
                    }
                }
            }
        }
    }

    let base_camps = scan_base_camps(&level_bytes);
    let guilds = scan_guilds(&level_bytes);

    let output = OutputJson {
        status: "success".to_string(),
        world_path: world_path.to_string_lossy().into_owned(),
        game_mode: detect_game_mode(world_path),
        players,
        base_camps,
        guilds,
    };

    if is_json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        print_beautiful_report(&output);
    }
}

pub fn print_beautiful_report(output: &OutputJson) {
    let title = i18n::t("report_title");
    let border = "=".repeat(title.len() + 16);
    println!("{}", border);
    println!("        {}        ", title);
    println!("{}", border);
    println!("{}: {}", i18n::t("world_save_path"), output.world_path);
    println!(
        "{}: {}",
        i18n::t("game_mode_label"),
        i18n::t(&output.game_mode)
    );
    println!();

    for player in &output.players {
        let profile_label = format!("{}: {}", i18n::t("player_profile"), player.nickname);
        let separator = "-".repeat(profile_label.len() + 4);
        println!("{}", separator);
        println!("  {}  ", profile_label);
        println!("{}", separator);
        println!("  * {:30} : {}", i18n::t("player_uid"), player.player_uid);
        println!("  * {:30} : {}", i18n::t("instance_id"), player.instance_id);
        println!("  * {:30} : {}", i18n::t("level"), player.level);
        println!("  * {:30} : {}", i18n::t("current_experience"), player.exp);
        println!("  * {:30} : {:.2}", i18n::t("health_hp"), player.hp);
        println!(
            "  * {:30} : {:.2} / 100",
            i18n::t("satiety_stomach"),
            player.full_stomach
        );
        println!(
            "  * {:30} : {}",
            i18n::t("physical_status"),
            i18n::t(&player.physical_health)
        );
        println!(
            "  * {:30} : {}",
            i18n::t("tech_points_available"),
            player.technology_points
        );
        println!();

        println!("  [{}]", i18n::t("appearance_customization"));
        println!(
            "  * {:30} : {}",
            i18n::t("body_type"),
            i18n::t(
                player
                    .customization
                    .get("BodyMeshName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
            )
        );
        println!(
            "  * {:30} : {}",
            i18n::t("head_model"),
            player
                .customization
                .get("HeadMeshName")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
        );
        println!(
            "  * {:30} : {}",
            i18n::t("hair_model"),
            player
                .customization
                .get("HairMeshName")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
        );
        println!(
            "  * {:30} : {}",
            i18n::t("voice_selection_id"),
            player
                .customization
                .get("VoiceID")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        );
        println!();

        println!("  [{}]", i18n::t("active_quest_progression"));
        println!(
            "  * {:30} : {}",
            i18n::t("active_quest"),
            i18n::t(&player.active_quest)
        );
        let completed_translated: Vec<String> =
            player.completed_quests.iter().map(|q| i18n::t(q)).collect();
        println!(
            "  * {:30} : {}",
            i18n::t("completed_quests"),
            completed_translated.join(", ")
        );
        println!();

        println!("  [{}]", i18n::t("exploration_discovery"));
        println!(
            "  * {:30} : {}",
            i18n::t("relics_found"),
            player.relics_found
        );
        println!(
            "  * {:30} : {}",
            i18n::t("notes_read"),
            player.notes_found.join(", ")
        );
        println!(
            "  * {:30} : {} {}",
            i18n::t("fast_travel_unlocked"),
            player.fast_travel_points.len(),
            if player.fast_travel_points.len() == 1 {
                "point"
            } else {
                "points"
            }
        );
        println!();

        println!(
            "  [{} ({})]",
            i18n::t("active_team_pals"),
            player.active_pals.len()
        );
        if player.active_pals.is_empty() {
            println!("    {}", i18n::t("no_pals_in_team"));
        } else {
            for (idx, pal) in player.active_pals.iter().enumerate() {
                let gender_str = if pal.gender.contains("Female") {
                    "Female"
                } else {
                    "Male"
                };
                println!(
                    "    {}. {} [{} {}] ({})",
                    idx + 1,
                    i18n::t(&pal.character_id),
                    i18n::t("level"),
                    pal.level,
                    i18n::t(gender_str)
                );
                println!(
                    "       - HP                 : {:.2} / {:.2}",
                    pal.hp, pal.max_hp
                );
                println!("       - Satiety (Stomach)  : {:.2}", pal.satiety);
                println!(
                    "       - Status             : {}",
                    i18n::t(&pal.physical_health)
                );
                println!("       - Friendship         : {}", pal.friendship);

                let talents_str = format!(
                    "HP: {}, Atk: {}, Def: {}",
                    pal.talents.get("HP").unwrap_or(&0),
                    pal.talents.get("Shot").unwrap_or(&0),
                    pal.talents.get("Defense").unwrap_or(&0)
                );
                println!("       - {:18} : {}", i18n::t("talents"), talents_str);

                if !pal.passive_skills.is_empty() {
                    let passives_str = pal
                        .passive_skills
                        .iter()
                        .map(|p| i18n::t(p))
                        .collect::<Vec<String>>()
                        .join(", ");
                    println!(
                        "       - {:18} : {}",
                        i18n::t("passive_skills"),
                        passives_str
                    );
                }
            }
        }
        println!();

        println!(
            "  [{} ({})]",
            i18n::t("palbox_pals"),
            player.palbox_pals.len()
        );
        if player.palbox_pals.is_empty() {
            println!("    {}", i18n::t("no_pals_in_palbox"));
        } else {
            for (idx, pal) in player.palbox_pals.iter().enumerate() {
                let gender_str = if pal.gender.contains("Female") {
                    "Female"
                } else {
                    "Male"
                };
                println!(
                    "    {}. {} [{} {}] ({})",
                    idx + 1,
                    i18n::t(&pal.character_id),
                    i18n::t("level"),
                    pal.level,
                    i18n::t(gender_str)
                );
                println!(
                    "       - HP                 : {:.2} / {:.2}",
                    pal.hp, pal.max_hp
                );
                println!("       - Satiety (Stomach)  : {:.2}", pal.satiety);
                println!(
                    "       - Status             : {}",
                    i18n::t(&pal.physical_health)
                );
                println!("       - Friendship         : {}", pal.friendship);

                let talents_str = format!(
                    "HP: {}, Atk: {}, Def: {}",
                    pal.talents.get("HP").unwrap_or(&0),
                    pal.talents.get("Shot").unwrap_or(&0),
                    pal.talents.get("Defense").unwrap_or(&0)
                );
                println!("       - {:18} : {}", i18n::t("talents"), talents_str);

                if !pal.passive_skills.is_empty() {
                    let passives_str = pal
                        .passive_skills
                        .iter()
                        .map(|p| i18n::t(p))
                        .collect::<Vec<String>>()
                        .join(", ");
                    println!(
                        "       - {:18} : {}",
                        i18n::t("passive_skills"),
                        passives_str
                    );
                }
            }
        }
        println!();

        println!("  [{}]", i18n::t("equipped_weapons"));
        if player.weapons.is_empty() {
            println!("    {}", i18n::t("no_weapons_equipped"));
        } else {
            for item in &player.weapons {
                println!(
                    "    * Slot {}: {} (x{})",
                    item.slot_index, item.item_id, item.count
                );
            }
        }
        println!();

        println!("  [{}]", i18n::t("equipped_armor"));
        if player.armor.is_empty() {
            println!("    {}", i18n::t("no_armor_equipped"));
        } else {
            for item in &player.armor {
                println!(
                    "    * Slot {}: {} (x{})",
                    item.slot_index, item.item_id, item.count
                );
            }
        }
        println!();

        println!("  [{}]", i18n::t("backpack_inventory"));
        if player.common_inventory.is_empty() {
            println!("    {}", i18n::t("backpack_empty"));
        } else {
            for item in &player.common_inventory {
                println!(
                    "    * Slot {}: {} (x{})",
                    item.slot_index, item.item_id, item.count
                );
            }
        }
        println!();
    }

    println!("  [{}]", i18n::t("base_camps"));
    if output.base_camps.is_empty() {
        println!("    {}", i18n::t("no_base_camps_found"));
    } else {
        for camp in &output.base_camps {
            println!("    * Base Camp ID : {}", camp.base_camp_id);
            println!("      - Level      : {}", camp.level);
            println!("      - Guild ID   : {}", camp.group_id);
            println!(
                "      - Coordinates: ({:.1}, {:.1}, {:.1})",
                camp.coordinates.0, camp.coordinates.1, camp.coordinates.2
            );
        }
    }
    println!();

    println!("  [{}]", i18n::t("guilds"));
    if output.guilds.is_empty() {
        println!("    {}", i18n::t("no_guilds_found"));
    } else {
        for guild in &output.guilds {
            println!("    * Guild Name   : {}", guild.guild_name);
            println!("      - Guild ID   : {}", guild.guild_id);
            println!("      - Leader UID : {}", guild.admin_player_uid);
            println!("      - Members    : {}", guild.members.join(", "));
        }
    }
    println!();

    println!("{}", border);
}
