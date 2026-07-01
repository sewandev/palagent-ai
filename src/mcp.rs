use crate::i18n;
use crate::server::{execute_command_captured, execute_command_remote};
use std::io::{self, BufRead};
use std::path::PathBuf;

pub fn run_mcp_loop(
    world_path: Option<PathBuf>,
    client_conn: Option<(String, String, Option<String>)>,
) {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    while let Ok(n) = handle.read_line(&mut line) {
        if n == 0 {
            break;
        }
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            line.clear();
            continue;
        }

        if let Ok(req) = serde_json::from_str::<serde_json::Value>(line_trimmed) {
            let id = req.get("id").cloned();
            let method = req
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or_default();

            match method {
                "initialize" => {
                    let resp = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "capabilities": {
                                "tools": {}
                            },
                            "protocolVersion": "2024-11-05",
                            "serverInfo": {
                                "name": "palagent-ai",
                                "version": "0.0.1"
                            }
                        }
                    });
                    println!("{}", serde_json::to_string(&resp).unwrap());
                }
                "tools/list" => {
                    let resp = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "tools": [
                                {
                                    "name": "list_worlds",
                                    "description": "List all detected Palworld save worlds and their paths",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {}
                                    }
                                },
                                {
                                    "name": "query_time",
                                    "description": "Get current in-game day, time, and cycle (day/night)",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {}
                                    }
                                },
                                {
                                    "name": "query_settings",
                                    "description": "Get server configuration and game difficulty settings",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {}
                                    }
                                },
                                {
                                    "name": "search_chest",
                                    "description": "Locate specific items across all base chests",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "query": {
                                                "type": "string",
                                                "description": "Item name to search (e.g. Berries, Wood)"
                                            }
                                        },
                                        "required": ["query"]
                                    }
                                },
                                {
                                    "name": "query_breeding",
                                    "description": "Analyze available gender combos and potential breeding offspring",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "player_uid": {
                                                "type": "string",
                                                "description": "Optional Player UID to isolate breeding team"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "query_progress",
                                    "description": "Check player notes found, fast travel unlocks, and capture progress",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "player_uid": {
                                                "type": "string",
                                                "description": "Optional Player UID to isolate progress"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "monitor_pals",
                                    "description": "Get real-time sanity (SAN), satiety (hunger), and HP levels of base/active Pals",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "player_uid": {
                                                "type": "string",
                                                "description": "Optional Player UID to isolate monitored Pals"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "query_analyzer",
                                    "description": "Analyze Pal talent IV stats (HP/Atk/Def bonuses) and passive skills",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "player_uid": {
                                                "type": "string",
                                                "description": "Optional Player UID to isolate Pals"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "query_full",
                                    "description": "Retrieve the complete world telemetry report including bases, players, and guilds",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "player_uid": {
                                                "type": "string",
                                                "description": "Optional Player UID to isolate report details"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "query_recipes",
                                    "description": "Query crafting recipes for items like Pal Spheres",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "item_id": {
                                                "type": "string",
                                                "description": "Optional Item ID to query (e.g. palsphere, palsphere_mega, palsphere_giga)"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "query_active_skills",
                                    "description": "Query combat active skill stats like power, cooldown, and element",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "skill_id": {
                                                "type": "string",
                                                "description": "Optional Skill ID to query (e.g. AirCanon, HydroLaser, FireBlast)"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "query_target_breeding",
                                    "description": "Query all parent combinations that produce a specific child Pal",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "target_pal": {
                                                "type": "string",
                                                "description": "Target child Pal name or internal ID (e.g. Anubis, Jetragon)"
                                            }
                                        },
                                        "required": ["target_pal"]
                                    }
                                },
                                {
                                    "name": "query_drops",
                                    "description": "Query drops of a Pal or locate which Pals drop a specific item",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "pal_name": {
                                                "type": "string",
                                                "description": "Pal name to query drops for (e.g. Lamball, Foxsparks)"
                                            },
                                            "item_name": {
                                                "type": "string",
                                                "description": "Item name to query dropping Pals (e.g. wool, flame_organ)"
                                            }
                                        }
                                    }
                                },
                                {
                                    "name": "calculate_capture_rate",
                                    "description": "Calculate capture rate percentages based on creature level, HP, sphere types, and Lifmunk statue level",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "pal_level": {
                                                "type": "integer",
                                                "description": "Creature level"
                                            },
                                            "current_hp": {
                                                "type": "integer",
                                                "description": "Optional current HP of the creature (default 100)"
                                            },
                                            "max_hp": {
                                                "type": "integer",
                                                "description": "Optional maximum HP of the creature (default 100)"
                                            },
                                            "lifmunk_level": {
                                                "type": "integer",
                                                "description": "Optional player Lifmunk capture bonus level (0 to 10)"
                                            }
                                        },
                                        "required": ["pal_level"]
                                    }
                                }
                            ]
                        }
                    });
                    println!("{}", serde_json::to_string(&resp).unwrap());
                }
                "tools/call" => {
                    let params = req
                        .get("params")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    let tool_name = params
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or_default();
                    let arguments = params
                        .get("arguments")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));

                    let player_uid = arguments.get("player_uid").and_then(|u| u.as_str());
                    let search_query = arguments
                        .get("query")
                        .and_then(|q| q.as_str())
                        .unwrap_or_default();

                    let text_result = if tool_name == "query_recipes" {
                        let item_id = arguments
                            .get("item_id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("");
                        let use_es = i18n::current_language() == i18n::Language::Es;
                        if item_id.is_empty() {
                            if use_es {
                                "Objetos conocidos con recetas: palsphere (Esfera Pal), palsphere_mega (Megaesfera), palsphere_giga (Gigaesfera)".to_string()
                            } else {
                                "Known items with recipes: palsphere, palsphere_mega, palsphere_giga".to_string()
                            }
                        } else {
                            let recipe = crate::db::get_recipe(item_id);
                            if recipe.is_empty() {
                                if use_es {
                                    format!("No se encontró receta para: {}", item_id)
                                } else {
                                    format!("No recipe found for: {}", item_id)
                                }
                            } else {
                                let translated_item = crate::db::translate_item(item_id, use_es);
                                let mut out = if use_es {
                                    format!("Receta para {}:\n", translated_item)
                                } else {
                                    format!("Recipe for {}:\n", translated_item)
                                };
                                for (ing, cnt) in recipe {
                                    let translated_ing = crate::db::translate_item(&ing, use_es);
                                    out.push_str(&format!("  - {}: {}\n", translated_ing, cnt));
                                }
                                out
                            }
                        }
                    } else if tool_name == "query_active_skills" {
                        let skill_id = arguments
                            .get("skill_id")
                            .and_then(|s| s.as_str())
                            .unwrap_or("");
                        let use_es = i18n::current_language() == i18n::Language::Es;
                        if skill_id.is_empty() {
                            if use_es {
                                "Habilidades conocidas: AirCanon, HydroLaser, DragonBreath, DarkLaser, FireBlast, WindCutter, AquaGun, ElectroBall, SandBlast, IceMissile".to_string()
                            } else {
                                "Known skills: AirCanon, HydroLaser, DragonBreath, DarkLaser, FireBlast, WindCutter, AquaGun, ElectroBall, SandBlast, IceMissile".to_string()
                            }
                        } else {
                            if let Some((name, power, cd, element)) =
                                crate::db::translate_active_skill(skill_id, use_es)
                            {
                                if use_es {
                                    format!("Habilidad: {}\n  - Poder: {}\n  - Tiempo de Recarga: {}s\n  - Elemento: {}", name, power, cd, element)
                                } else {
                                    format!("Skill: {}\n  - Power: {}\n  - Cooldown: {}s\n  - Element: {}", name, power, cd, element)
                                }
                            } else {
                                if use_es {
                                    format!("Habilidad desconocida: {}", skill_id)
                                } else {
                                    format!("Unknown skill: {}", skill_id)
                                }
                            }
                        }
                    } else if tool_name == "query_target_breeding" {
                        let target_pal = arguments
                            .get("target_pal")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        let use_es = i18n::current_language() == i18n::Language::Es;
                        if target_pal.is_empty() {
                            if use_es {
                                "Error: Debes proporcionar el nombre del Pal objetivo (target_pal)."
                                    .to_string()
                            } else {
                                "Error: You must provide a target child Pal name (target_pal)."
                                    .to_string()
                            }
                        } else {
                            let combos = crate::db::find_breeding_parents_for_target(target_pal);
                            if combos.is_empty() {
                                if use_es {
                                    format!("No se encontraron combinaciones de crianza para obtener a '{}'.", target_pal)
                                } else {
                                    format!(
                                        "No breeding combinations found to produce '{}'.",
                                        target_pal
                                    )
                                }
                            } else {
                                let target_translated =
                                    crate::db::translate_pal(target_pal, use_es);
                                let mut out = if use_es {
                                    format!(
                                        "Parejas para criar a {} ({} combinaciones):\n",
                                        target_translated,
                                        combos.len()
                                    )
                                } else {
                                    format!(
                                        "Parents to breed {} ({} combinations):\n",
                                        target_translated,
                                        combos.len()
                                    )
                                };
                                for (pa, pb) in combos {
                                    let trans_a = crate::db::translate_pal(&pa, use_es);
                                    let trans_b = crate::db::translate_pal(&pb, use_es);
                                    out.push_str(&format!("  - {} + {}\n", trans_a, trans_b));
                                }
                                out
                            }
                        }
                    } else if tool_name == "query_drops" {
                        let pal_name = arguments
                            .get("pal_name")
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let item_name = arguments
                            .get("item_name")
                            .and_then(|i| i.as_str())
                            .unwrap_or("");
                        let use_es = i18n::current_language() == i18n::Language::Es;

                        if !pal_name.is_empty() {
                            let drops = crate::db::get_pal_drops(pal_name);
                            if drops.is_empty() {
                                if use_es {
                                    format!("No se registraron drops para el Pal: {}", pal_name)
                                } else {
                                    format!("No drops registered for Pal: {}", pal_name)
                                }
                            } else {
                                let pal_translated = crate::db::translate_pal(pal_name, use_es);
                                let mut out = if use_es {
                                    format!("Drops de {}:\n", pal_translated)
                                } else {
                                    format!("Drops for {}:\n", pal_translated)
                                };
                                for (item, chance, min_q, max_q) in drops {
                                    let trans_item = crate::db::translate_item(&item, use_es);
                                    out.push_str(&format!(
                                        "  - {}: {}% (Cant: {}-{})\n",
                                        trans_item, chance, min_q, max_q
                                    ));
                                }
                                out
                            }
                        } else if !item_name.is_empty() {
                            let pals = crate::db::get_pals_dropping_item(item_name);
                            if pals.is_empty() {
                                if use_es {
                                    format!(
                                        "No se encontraron Pals que suelten el objeto: {}",
                                        item_name
                                    )
                                } else {
                                    format!("No Pals found dropping item: {}", item_name)
                                }
                            } else {
                                let item_translated = crate::db::translate_item(item_name, use_es);
                                let mut out = if use_es {
                                    format!("Pals que sueltan {}:\n", item_translated)
                                } else {
                                    format!("Pals dropping {}:\n", item_translated)
                                };
                                for pal in pals {
                                    let trans_pal = crate::db::translate_pal(&pal, use_es);
                                    out.push_str(&format!("  - {}\n", trans_pal));
                                }
                                out
                            }
                        } else {
                            if use_es {
                                "Error: Proporciona pal_name o item_name para consultar drops."
                                    .to_string()
                            } else {
                                "Error: Provide either pal_name or item_name to query drops."
                                    .to_string()
                            }
                        }
                    } else if tool_name == "calculate_capture_rate" {
                        let pal_level = arguments
                            .get("pal_level")
                            .and_then(|l| l.as_i64())
                            .unwrap_or(1) as i32;
                        let current_hp = arguments
                            .get("current_hp")
                            .and_then(|h| h.as_i64())
                            .unwrap_or(100) as i32;
                        let max_hp = arguments
                            .get("max_hp")
                            .and_then(|m| m.as_i64())
                            .unwrap_or(100) as i32;
                        let lifmunk_level = arguments
                            .get("lifmunk_level")
                            .and_then(|l| l.as_i64())
                            .unwrap_or(0) as i32;
                        let use_es = i18n::current_language() == i18n::Language::Es;

                        let spheres = [
                            ("palsphere", "Común / Common"),
                            ("palsphere_mega", "Mega Esfera"),
                            ("palsphere_giga", "Giga Esfera"),
                            ("palsphere_tera", "Tera Esfera"),
                            ("palsphere_ultra", "Ultra Esfera"),
                            ("palsphere_legendary", "Legendaria / Legendary"),
                        ];

                        let mut out = if use_es {
                            format!("Probabilidades estimadas de captura para un Pal Nvl {} (HP: {}/{} | Bonus Estatua Nvl {}):\n", pal_level, current_hp, max_hp, lifmunk_level)
                        } else {
                            format!("Estimated capture rates for a Lvl {} Pal (HP: {}/{} | Statue Bonus Lvl {}):\n", pal_level, current_hp, max_hp, lifmunk_level)
                        };

                        for (sphere_id, sphere_name) in spheres {
                            let rate = crate::db::calculate_capture_rate(
                                pal_level,
                                current_hp,
                                max_hp,
                                sphere_id,
                                lifmunk_level,
                            );
                            out.push_str(&format!("  - {}: {:.1}%\n", sphere_name, rate));
                        }
                        out
                    } else if let Some((host, passcode, default_uid)) = &client_conn {
                        let target_uid = player_uid.or(default_uid.as_deref());
                        let cmd = match tool_name {
                            "list_worlds" => "list-worlds".to_string(),
                            "query_time" => "time".to_string(),
                            "query_settings" => "settings".to_string(),
                            "search_chest" => format!("search-chest:{}", search_query),
                            "query_breeding" => "breeding".to_string(),
                            "query_progress" => "progress".to_string(),
                            "monitor_pals" => "monitor".to_string(),
                            "query_analyzer" => "analyzer".to_string(),
                            "query_full" => "full".to_string(),
                            _ => "".to_string(),
                        };
                        if cmd.is_empty() {
                            format!("Unknown tool: {}", tool_name)
                        } else {
                            execute_command_remote(host, passcode, &cmd, target_uid)
                        }
                    } else {
                        let path = world_path.as_ref().unwrap();
                        match tool_name {
                            "list_worlds" => {
                                execute_command_captured(path, "list-worlds", false, None)
                            }
                            "query_time" => execute_command_captured(path, "time", false, None),
                            "query_settings" => {
                                execute_command_captured(path, "settings", false, None)
                            }
                            "search_chest" => execute_command_captured(
                                path,
                                &format!("search-chest:{}", search_query),
                                false,
                                None,
                            ),
                            "query_breeding" => {
                                execute_command_captured(path, "breeding", false, player_uid)
                            }
                            "query_progress" => {
                                execute_command_captured(path, "progress", false, player_uid)
                            }
                            "monitor_pals" => {
                                execute_command_captured(path, "monitor", false, player_uid)
                            }
                            "query_analyzer" => {
                                execute_command_captured(path, "analyzer", false, player_uid)
                            }
                            "query_full" => {
                                execute_command_captured(path, "full", false, player_uid)
                            }
                            _ => format!("Unknown tool: {}", tool_name),
                        }
                    };

                    let resp = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": text_result
                                }
                            ]
                        }
                    });
                    println!("{}", serde_json::to_string(&resp).unwrap());
                }
                _ => {
                    if id.is_some() {
                        let resp = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32601,
                                "message": format!("Method not found: {}", method)
                            }
                        });
                        println!("{}", serde_json::to_string(&resp).unwrap());
                    }
                }
            }
        }

        line.clear();
    }
}
