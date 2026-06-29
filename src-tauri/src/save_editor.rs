use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInfo {
    pub player_uid: String,
    pub nickname: String,
    pub common_container_id: String,
    pub weapon_container_id: String,
    pub armor_container_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryItem {
    pub slot_index: usize,
    pub item_id: String,
    pub count: u32,
}

// Structs for parsing Player Save JSON files
#[derive(Deserialize, Debug)]
struct PlayerSaveJson {
    properties: PlayerSaveProperties,
}

#[derive(Deserialize, Debug)]
struct PlayerSaveProperties {
    SaveData: PlayerSaveDataProperty,
}

#[derive(Deserialize, Debug)]
struct PlayerSaveDataProperty {
    value: PlayerSaveDataValue,
}

#[derive(Deserialize, Debug)]
struct PlayerSaveDataValue {
    InventoryInfo: PlayerInventoryInfoProperty,
}

#[derive(Deserialize, Debug)]
struct PlayerInventoryInfoProperty {
    value: PlayerInventoryInfoValue,
}

#[derive(Deserialize, Debug)]
struct PlayerInventoryInfoValue {
    CommonContainerId: ContainerIdProperty,
    WeaponLoadOutContainerId: ContainerIdProperty,
    PlayerEquipArmorContainerId: ContainerIdProperty,
}

#[derive(Deserialize, Debug)]
struct ContainerIdProperty {
    value: ContainerIdValue,
}

#[derive(Deserialize, Debug)]
struct ContainerIdValue {
    ID: GuidProperty,
}

#[derive(Deserialize, Debug)]
struct GuidProperty {
    value: String,
}

// Convert a hex string UID from filename to a hyphenated GUID (e.g. 00000000000000000000000000000001 -> 00000000-0000-0000-0000-000000000001)
fn hex_to_guid(hex: &str) -> String {
    if hex.len() != 32 {
        return hex.to_string();
    }
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

// Scans CharacterSaveParameterMap binary data to extract NickName
fn extract_nickname(bytes: &[u8]) -> Option<String> {
    let nickname_pat = b"\x09\x00\x00\x00NickName\x00";
    let str_prop_pat = b"\x0c\x00\x00\x00StrProperty\x00";

    // Find the nickname pattern
    let idx = bytes.windows(nickname_pat.len()).position(|w| w == nickname_pat)?;
    
    // Find the StrProperty pattern after the nickname pattern
    let str_idx = bytes[idx..].windows(str_prop_pat.len()).position(|w| w == str_prop_pat)?;
    let absolute_str_idx = idx + str_idx;

    // Skip StrProperty + size (8 bytes) + metadata (1 byte) = 16 + 8 + 1 = 25 bytes
    let val_idx = absolute_str_idx + str_prop_pat.len() + 8 + 1;
    if val_idx + 4 <= bytes.len() {
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&bytes[val_idx..val_idx + 4]);
        let str_len = u32::from_le_bytes(len_bytes) as usize;

        if str_len > 0 && str_len < 100 && val_idx + 4 + str_len <= bytes.len() {
            let name_bytes = &bytes[val_idx + 4..val_idx + 4 + str_len - 1]; // Omit null byte
            return String::from_utf8(name_bytes.to_vec()).ok();
        }
    }

    None
}

// Scans Level.sav JSON file to retrieve all player UIDs and their nicknames
pub fn get_player_nicknames(level_json_path: &str) -> Result<std::collections::HashMap<String, String>, String> {
    let content = fs::read_to_string(level_json_path)
        .map_err(|e| format!("Failed to read Level JSON file: {}", e))?;

    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse Level JSON: {}", e))?;

    let mut nicknames = std::collections::HashMap::new();

    if let Some(character_map) = v.pointer("/properties/worldSaveData/value/CharacterSaveParameterMap/value") {
        if let Some(arr) = character_map.as_array() {
            for char_entry in arr {
                if let (Some(player_uid_val), Some(raw_data_val)) = (
                    char_entry.pointer("/key/PlayerUId/value"),
                    char_entry.pointer("/value/RawData/value/values")
                ) {
                    if let (Some(uid_str), Some(raw_bytes_arr)) = (player_uid_val.as_str(), raw_data_val.as_array()) {
                        let bytes: Vec<u8> = raw_bytes_arr
                            .iter()
                            .filter_map(|x| x.as_u64().map(|y| y as u8))
                            .collect();

                        if let Some(nickname) = extract_nickname(&bytes) {
                            nicknames.insert(uid_str.to_string(), nickname);
                        }
                    }
                }
            }
        }
    }

    Ok(nicknames)
}

// Scans all player save files and matches them with nicknames to retrieve PlayerInfo list
pub fn scan_players(save_dir: &str, level_json_path: &str) -> Result<Vec<PlayerInfo>, String> {
    let players_dir = Path::new(save_dir).join("Players");
    if !players_dir.exists() {
        return Err("Players directory not found in the save path".to_string());
    }

    let nicknames = get_player_nicknames(level_json_path)?;
    let mut player_infos = Vec::new();

    // Iterate over player .json files (assumed already converted to JSON for parsing)
    for entry in fs::read_dir(&players_dir).map_err(|e| format!("Failed to read Players dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read Players dir entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let player_uid = hex_to_guid(file_stem);

            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read player JSON {}: {}", file_stem, e))?;

            if let Ok(player_json) = serde_json::from_str::<PlayerSaveJson>(&content) {
                let inventory = player_json.properties.SaveData.value.InventoryInfo.value;
                let nickname = nicknames.get(&player_uid)
                    .cloned()
                    .unwrap_or_else(|| {
                        if file_stem == "00000000000000000000000000000001" {
                            "Host Player".to_string()
                        } else {
                            format!("Player_{}", &file_stem[0..8])
                        }
                    });

                player_infos.push(PlayerInfo {
                    player_uid,
                    nickname,
                    common_container_id: inventory.CommonContainerId.value.ID.value,
                    weapon_container_id: inventory.WeaponLoadOutContainerId.value.ID.value,
                    armor_container_id: inventory.PlayerEquipArmorContainerId.value.ID.value,
                });
            }
        }
    }

    Ok(player_infos)
}

// Parses raw binary bytes of a single slot to extract ItemId and count
fn parse_slot(slot_index: usize, bytes: &[u8]) -> Option<InventoryItem> {
    if bytes.len() < 12 {
        return None;
    }

    let count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let id_length = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;

    if id_length == 0 || id_length > 100 || bytes.len() < 12 + id_length {
        return None;
    }

    let item_id_bytes = &bytes[12..12 + id_length - 1]; // Omit null byte
    let item_id = String::from_utf8(item_id_bytes.to_vec()).unwrap_or_default();

    if item_id.is_empty() || count == 0 {
        return None;
    }

    Some(InventoryItem {
        slot_index,
        item_id,
        count,
    })
}

// Loads all items inside a container by its GUID
pub fn get_container_items(level_json_path: &str, container_guid: &str) -> Result<Vec<InventoryItem>, String> {
    let content = fs::read_to_string(level_json_path)
        .map_err(|e| format!("Failed to read Level JSON: {}", e))?;

    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse Level JSON: {}", e))?;

    let mut items = Vec::new();

    if let Some(containers) = v.pointer("/properties/worldSaveData/value/ItemContainerSaveData/value") {
        if let Some(arr) = containers.as_array() {
            for container in arr {
                if let Some(guid_val) = container.pointer("/key/ID/value") {
                    if guid_val.as_str() == Some(container_guid) {
                        if let Some(slots) = container.pointer("/value/Slots/value/values") {
                            if let Some(slots_arr) = slots.as_array() {
                                for (idx, slot) in slots_arr.iter().enumerate() {
                                    if let Some(raw_data_arr) = slot.pointer("/RawData/value/values") {
                                        if let Some(raw_bytes_val) = raw_data_arr.as_array() {
                                            let bytes: Vec<u8> = raw_bytes_val
                                                .iter()
                                                .filter_map(|x| x.as_u64().map(|y| y as u8))
                                                .collect();

                                            if let Some(item) = parse_slot(idx, &bytes) {
                                                items.push(item);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(items)
}

// Modifies a single item slot inside a container by its GUID and slot index
pub fn modify_container_item(
    level_json_path: &str,
    container_guid: &str,
    slot_index: usize,
    item_id: &str,
    count: u32,
) -> Result<(), String> {
    let content = fs::read_to_string(level_json_path)
        .map_err(|e| format!("Failed to read Level JSON: {}", e))?;

    let mut v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse Level JSON: {}", e))?;

    let mut modified = false;

    if let Some(containers) = v.pointer_mut("/properties/worldSaveData/value/ItemContainerSaveData/value") {
        if let Some(arr) = containers.as_array_mut() {
            for container in arr {
                if let Some(guid_val) = container.pointer("/key/ID/value") {
                    if guid_val.as_str() == Some(container_guid) {
                        if let Some(slots) = container.pointer_mut("/value/Slots/value/values") {
                            if let Some(slots_arr) = slots.as_array_mut() {
                                if slot_index < slots_arr.len() {
                                    let slot = &mut slots_arr[slot_index];
                                    if let Some(raw_data_arr) = slot.pointer_mut("/RawData/value/values") {
                                        if let Some(raw_bytes_val) = raw_data_arr.as_array_mut() {
                                            // Get the existing bytes as a mutable Vec<u8>
                                            let mut bytes: Vec<u8> = raw_bytes_val
                                                .iter()
                                                .filter_map(|x| x.as_u64().map(|y| y as u8))
                                                .collect();

                                            if item_id.is_empty() || count == 0 {
                                                // Clear item: set stack count and ID length to 0
                                                if bytes.len() >= 12 {
                                                    // Zero out stack count (bytes 4-7)
                                                    for i in 4..8 {
                                                        bytes[i] = 0;
                                                    }
                                                    // Zero out ID length (bytes 8-11)
                                                    for i in 8..12 {
                                                        bytes[i] = 0;
                                                    }
                                                    // Zero out the rest of the bytes
                                                    for i in 12..bytes.len() {
                                                        bytes[i] = 0;
                                                    }
                                                }
                                            } else {
                                                // Reconstruct the slot raw bytes
                                                let mut new_bytes = vec![0u8; 12];
                                                
                                                // Preserve bytes 0-3 (slot index or default)
                                                if bytes.len() >= 4 {
                                                    new_bytes[0..4].copy_from_slice(&bytes[0..4]);
                                                }
                                                
                                                // Bytes 4-7: stack count (little endian)
                                                let count_bytes = count.to_le_bytes();
                                                new_bytes[4..8].copy_from_slice(&count_bytes);

                                                // Bytes 8-11: ID length (string len + 1 null byte)
                                                let id_len = item_id.len() + 1;
                                                let id_len_bytes = (id_len as u32).to_le_bytes();
                                                new_bytes[8..12].copy_from_slice(&id_len_bytes);

                                                // Append item_id string and null byte
                                                new_bytes.extend_from_slice(item_id.as_bytes());
                                                new_bytes.push(0x00);

                                                // Preserve any extra trailing bytes (durability, custom data) if present in original slot
                                                if bytes.len() >= 12 {
                                                    let original_id_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
                                                    let original_str_end = 12 + original_id_len;
                                                    if original_str_end < bytes.len() {
                                                        new_bytes.extend_from_slice(&bytes[original_str_end..]);
                                                    }
                                                }
                                                
                                                bytes = new_bytes;
                                            }

                                            // Write bytes back to the JSON structure
                                            *raw_bytes_val = bytes
                                                .iter()
                                                .map(|&x| serde_json::Value::Number(serde_json::Number::from(x)))
                                                .collect();

                                            modified = true;
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    if modified {
        let new_content = serde_json::to_string_pretty(&v)
            .map_err(|e| format!("Failed to serialize Level JSON: {}", e))?;
        fs::write(level_json_path, new_content)
            .map_err(|e| format!("Failed to write modified Level JSON: {}", e))?;
        Ok(())
    } else {
        Err("Container GUID or slot index not found".to_string())
    }
}

