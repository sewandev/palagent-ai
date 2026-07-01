use crate::models::{BaseCampSummary, CharacterEntry, GuildSummary, InventoryItem};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

pub fn format_guid(bytes: &[u8; 16]) -> String {
    let mut swapped = [0u8; 16];
    for chunk in 0..4 {
        for byte in 0..4 {
            swapped[chunk * 4 + byte] = bytes[chunk * 4 + (3 - byte)];
        }
    }
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        swapped[0], swapped[1], swapped[2], swapped[3],
        swapped[4], swapped[5],
        swapped[6], swapped[7],
        swapped[8], swapped[9],
        swapped[10], swapped[11], swapped[12], swapped[13], swapped[14], swapped[15]
    )
}

pub fn read_string_at(bytes: &[u8], offset: &mut usize) -> String {
    if *offset + 4 > bytes.len() {
        return String::new();
    }
    let len = i32::from_le_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    if len == 0 {
        return String::new();
    }
    if len > 0 {
        let u_len = len as usize;
        if *offset + u_len > bytes.len() {
            return String::new();
        }
        let str_bytes = &bytes[*offset..*offset + u_len];
        *offset += u_len;
        let mut s = String::from_utf8_lossy(str_bytes).into_owned();
        if s.ends_with('\0') {
            s.pop();
        }
        s
    } else {
        let w_len = (-len) as usize;
        if *offset + w_len * 2 > bytes.len() {
            return String::new();
        }
        let char_bytes = &bytes[*offset..*offset + w_len * 2];
        *offset += w_len * 2;
        let chars: Vec<u16> = char_bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let mut s = String::from_utf16_lossy(&chars);
        if s.ends_with('\0') {
            s.pop();
        }
        s
    }
}

pub fn extract_guid_prop(bytes: &[u8], prop_name: &[u8]) -> Option<String> {
    extract_guid_bytes_prop(bytes, prop_name).map(|g| format_guid(&g))
}

pub fn extract_guid_bytes_prop(bytes: &[u8], prop_name: &[u8]) -> Option<[u8; 16]> {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = pos?;
    let guid_pat = b"Guid";
    let mut guid_pos = None;
    for i in pos..std::cmp::min(pos + 150, bytes.len()) {
        if i + guid_pat.len() <= bytes.len() && &bytes[i..i + guid_pat.len()] == guid_pat {
            guid_pos = Some(i);
            break;
        }
    }
    let guid_pos = guid_pos?;
    if guid_pos + 38 <= bytes.len() {
        let mut g_bytes = [0u8; 16];
        g_bytes.copy_from_slice(&bytes[guid_pos + 22..guid_pos + 38]);
        Some(g_bytes)
    } else {
        None
    }
}

pub fn extract_string_prop(bytes: &[u8], prop_name: &[u8]) -> String {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return String::new(),
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "StrProperty" && t != "NameProperty" {
        return String::new();
    }
    offset += 8; // skip size
    offset += 1; // separator
    read_string_at(bytes, &mut offset)
}

pub fn extract_int_prop(bytes: &[u8], prop_name: &[u8]) -> i32 {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return 0,
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "IntProperty" {
        return 0;
    }
    offset += 8; // skip size
    offset += 1; // separator
    if offset + 4 <= bytes.len() {
        i32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ])
    } else {
        0
    }
}

pub fn extract_int64_prop(bytes: &[u8], prop_name: &[u8]) -> u64 {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return 0,
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "Int64Property" {
        return 0;
    }
    offset += 8; // skip size
    offset += 1; // separator
    if offset + 8 <= bytes.len() {
        u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ])
    } else {
        0
    }
}

pub fn extract_byte_prop(bytes: &[u8], prop_name: &[u8]) -> u32 {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return 0,
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "ByteProperty" {
        return 0;
    }
    offset += 8; // skip size
    let enum_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator
    if enum_type == "None" {
        if offset < bytes.len() {
            bytes[offset] as u32
        } else {
            0
        }
    } else {
        0
    }
}

pub fn extract_float_prop(bytes: &[u8], prop_name: &[u8]) -> f64 {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return 0.0,
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "FloatProperty" {
        return 0.0;
    }
    offset += 8; // skip size
    offset += 1; // separator
    if offset + 4 <= bytes.len() {
        f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as f64
    } else {
        0.0
    }
}

pub fn extract_fixed_point_prop(bytes: &[u8], prop_name: &[u8]) -> f64 {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return 0.0,
    };
    let val_pat = b"Value\x00";
    let mut val_pos = None;
    for i in pos..std::cmp::min(pos + 120, bytes.len()) {
        if i + val_pat.len() <= bytes.len() && &bytes[i..i + val_pat.len()] == val_pat {
            val_pos = Some(i);
            break;
        }
    }
    let vpos = match val_pos {
        Some(p) => p,
        None => return 0.0,
    };
    let mut offset = vpos + val_pat.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "Int64Property" {
        return 0.0;
    }
    offset += 8; // skip size
    offset += 1; // separator
    if offset + 8 <= bytes.len() {
        let v = i64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        v as f64 / 1000.0
    } else {
        0.0
    }
}

pub fn extract_array_strings(bytes: &[u8], prop_name: &[u8]) -> Vec<String> {
    let mut results = Vec::new();
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return results,
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "ArrayProperty" {
        return results;
    }
    offset += 8; // skip size
    let elem_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator
    if offset + 4 > bytes.len() {
        return results;
    }
    let count = u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]) as usize;
    offset += 4;

    if elem_type == "StructProperty" {
        let _st_name = read_string_at(bytes, &mut offset);
        let _st_type = read_string_at(bytes, &mut offset);
        offset += 8; // skip size
        let _st_typename = read_string_at(bytes, &mut offset);
        offset += 16 + 1; // GUID + separator

        for _ in 0..count {
            if offset >= bytes.len() {
                break;
            }
            let q_name_pat = b"QuestName\x00";
            let mut q_pos = None;
            for i in offset..std::cmp::min(offset + 150, bytes.len()) {
                if i + q_name_pat.len() <= bytes.len()
                    && &bytes[i..i + q_name_pat.len()] == q_name_pat
                {
                    q_pos = Some(i);
                    break;
                }
            }
            if let Some(qp) = q_pos {
                let mut temp_off = qp + q_name_pat.len();
                let _prop_t = read_string_at(bytes, &mut temp_off);
                temp_off += 8; // size
                temp_off += 1; // separator
                let name = read_string_at(bytes, &mut temp_off);
                if !name.is_empty() {
                    results.push(name);
                }
            }
            let mut depth = 1;
            while offset < bytes.len() {
                let s = read_string_at(bytes, &mut offset);
                if s == "None" {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                } else if !s.is_empty() {
                    let prop_t = read_string_at(bytes, &mut offset);
                    let size = i64::from_le_bytes([
                        bytes[offset],
                        bytes[offset + 1],
                        bytes[offset + 2],
                        bytes[offset + 3],
                        bytes[offset + 4],
                        bytes[offset + 5],
                        bytes[offset + 6],
                        bytes[offset + 7],
                    ]) as usize;
                    offset += 8;
                    if prop_t == "StructProperty" {
                        let _st = read_string_at(bytes, &mut offset);
                        offset += 17;
                    } else {
                        offset += 1;
                    }
                    offset += size;
                } else {
                    offset += 1;
                }
            }
        }
    } else {
        for _ in 0..count {
            if offset >= bytes.len() {
                break;
            }
            let s = read_string_at(bytes, &mut offset);
            if !s.is_empty() {
                results.push(s);
            }
        }
    }
    results
}

pub fn extract_map_keys(bytes: &[u8], map_name: &[u8]) -> Vec<String> {
    let mut results = Vec::new();
    let mut pos = None;
    for i in 0..bytes.len() - map_name.len() {
        if &bytes[i..i + map_name.len()] == map_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return results,
    };
    let mut offset = pos + map_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "MapProperty" {
        return results;
    }
    offset += 8; // skip size
    let _key_type = read_string_at(bytes, &mut offset);
    let val_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator

    if offset + 8 > bytes.len() {
        return results;
    }
    let _keys_to_delete = u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]);
    offset += 4;
    let count = u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]) as usize;
    offset += 4;

    for _ in 0..count {
        if offset >= bytes.len() {
            break;
        }
        let key = read_string_at(bytes, &mut offset);
        if key.is_empty() {
            break;
        }
        results.push(key);
        if val_type == "BoolProperty" {
            offset += 1;
        } else if val_type == "IntProperty" {
            offset += 4;
        } else if val_type == "StructProperty" {
            offset += 16;
        } else {
            offset += 4;
        }
    }
    results
}

pub fn extract_map_counts(bytes: &[u8], map_name: &[u8]) -> HashMap<String, u32> {
    let mut results = HashMap::new();
    let mut pos = None;
    for i in 0..bytes.len() - map_name.len() {
        if &bytes[i..i + map_name.len()] == map_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return results,
    };
    let mut offset = pos + map_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "MapProperty" {
        return results;
    }
    offset += 8; // skip size
    let _key_type = read_string_at(bytes, &mut offset);
    let val_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator

    if offset + 8 > bytes.len() {
        return results;
    }
    let _keys_to_delete = u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]);
    offset += 4;
    let count = u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]) as usize;
    offset += 4;

    for _ in 0..count {
        if offset >= bytes.len() {
            break;
        }
        let key = read_string_at(bytes, &mut offset);
        if key.is_empty() {
            break;
        }
        let mut val = 0;
        if val_type == "IntProperty" {
            if offset + 4 <= bytes.len() {
                val = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                offset += 4;
            }
        } else {
            offset += 4;
        }
        results.insert(key, val);
    }
    results
}

pub fn skip_property_at(bytes: &[u8], offset: &mut usize, type_name: &str, size: usize) {
    if type_name == "StructProperty" {
        let _struct_type = read_string_at(bytes, offset);
        *offset += 16 + 1; // GUID + separator
    } else if type_name == "ArrayProperty" {
        let _array_type = read_string_at(bytes, offset);
        *offset += 1;
    } else if type_name == "MapProperty" {
        let _key_type = read_string_at(bytes, offset);
        let _val_type = read_string_at(bytes, offset);
        *offset += 1;
    } else if type_name == "BoolProperty" {
        *offset += 1 + 1;
        return;
    } else if type_name == "EnumProperty" {
        let _enum_type = read_string_at(bytes, offset);
        *offset += 1;
    } else {
        *offset += 1;
    }
    *offset += size;
}

pub fn has_prop(bytes: &[u8], prop_name: &[u8]) -> bool {
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            return true;
        }
    }
    false
}

pub fn extract_bool_prop(bytes: &[u8], prop_name: &[u8]) -> bool {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return false,
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "BoolProperty" {
        return false;
    }
    offset += 8; // skip size
    if offset + 2 <= bytes.len() {
        bytes[offset] != 0
    } else {
        false
    }
}

pub fn extract_enum_prop(bytes: &[u8], prop_name: &[u8]) -> String {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return String::new(),
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t != "EnumProperty" {
        return String::new();
    }
    offset += 8; // skip size
    let _enum_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator
    read_string_at(bytes, &mut offset)
}

pub fn scan_character_save_parameters(level_bytes: &[u8]) -> Vec<CharacterEntry> {
    let mut entries = Vec::new();
    let pattern = b"CharacterSaveParameterMap\x00";
    let mut pos = None;
    for i in 0..level_bytes.len() - pattern.len() {
        if &level_bytes[i..i + pattern.len()] == pattern {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return entries,
    };
    let mut offset = pos + pattern.len();

    let prop_type = read_string_at(level_bytes, &mut offset);
    if prop_type != "MapProperty" {
        return entries;
    }
    offset += 8;
    let _key_type = read_string_at(level_bytes, &mut offset);
    let _val_type = read_string_at(level_bytes, &mut offset);
    offset += 1;

    if offset + 8 > level_bytes.len() {
        return entries;
    }
    let _keys_to_delete = u32::from_le_bytes([
        level_bytes[offset],
        level_bytes[offset + 1],
        level_bytes[offset + 2],
        level_bytes[offset + 3],
    ]);
    offset += 4;
    let count = u32::from_le_bytes([
        level_bytes[offset],
        level_bytes[offset + 1],
        level_bytes[offset + 2],
        level_bytes[offset + 3],
    ]) as usize;
    offset += 4;

    for _ in 0..count {
        if offset >= level_bytes.len() {
            break;
        }
        let mut player_uid = String::new();
        let mut instance_id = String::new();

        loop {
            let p_name = read_string_at(level_bytes, &mut offset);
            if p_name.is_empty() || p_name == "None" {
                break;
            }
            let p_type = read_string_at(level_bytes, &mut offset);
            let size = i64::from_le_bytes([
                level_bytes[offset],
                level_bytes[offset + 1],
                level_bytes[offset + 2],
                level_bytes[offset + 3],
                level_bytes[offset + 4],
                level_bytes[offset + 5],
                level_bytes[offset + 6],
                level_bytes[offset + 7],
            ]) as usize;
            offset += 8;

            let mut struct_type = String::new();
            if p_type == "StructProperty" {
                struct_type = read_string_at(level_bytes, &mut offset);
                offset += 17;
            } else {
                offset += 1;
            }

            let val_offset = offset;
            offset += size;

            if p_name == "PlayerUId" && struct_type == "Guid" {
                let mut g_bytes = [0u8; 16];
                g_bytes.copy_from_slice(&level_bytes[val_offset..val_offset + 16]);
                player_uid = format_guid(&g_bytes);
            } else if p_name == "InstanceId" && struct_type == "Guid" {
                let mut g_bytes = [0u8; 16];
                g_bytes.copy_from_slice(&level_bytes[val_offset..val_offset + 16]);
                instance_id = format_guid(&g_bytes);
            }
        }

        let mut raw_data = Vec::new();
        loop {
            let p_name = read_string_at(level_bytes, &mut offset);
            if p_name.is_empty() || p_name == "None" {
                break;
            }
            let p_type = read_string_at(level_bytes, &mut offset);
            let size = i64::from_le_bytes([
                level_bytes[offset],
                level_bytes[offset + 1],
                level_bytes[offset + 2],
                level_bytes[offset + 3],
                level_bytes[offset + 4],
                level_bytes[offset + 5],
                level_bytes[offset + 6],
                level_bytes[offset + 7],
            ]) as usize;
            offset += 8;

            if p_type == "ArrayProperty" {
                let array_type = read_string_at(level_bytes, &mut offset);
                offset += 1;

                let val_offset = offset;
                offset += size;

                if p_name == "RawData" && array_type == "ByteProperty" {
                    let b_count = u32::from_le_bytes([
                        level_bytes[val_offset],
                        level_bytes[val_offset + 1],
                        level_bytes[val_offset + 2],
                        level_bytes[val_offset + 3],
                    ]) as usize;
                    if val_offset + 4 + b_count <= level_bytes.len() {
                        raw_data = level_bytes[val_offset + 4..val_offset + 4 + b_count].to_vec();
                    }
                }
            } else {
                if p_type == "StructProperty" {
                    let _st = read_string_at(level_bytes, &mut offset);
                    offset += 17;
                } else {
                    offset += 1;
                }
                offset += size;
            }
        }

        if !player_uid.is_empty() && !instance_id.is_empty() && !raw_data.is_empty() {
            entries.push(CharacterEntry {
                player_uid,
                instance_id,
                raw_data,
            });
        }
    }
    entries
}

pub fn parse_container_items(
    level_bytes: &[u8],
    container_guid_le: &[u8; 16],
) -> Vec<InventoryItem> {
    let mut level_container_pos = None;
    for i in 0..level_bytes.len() - 16 {
        if &level_bytes[i..i + 16] == container_guid_le {
            let check_end = std::cmp::min(i + 40, level_bytes.len());
            if level_bytes[i + 16..check_end]
                .windows(10)
                .any(|w| w == b"BelongInfo")
            {
                level_container_pos = Some(i);
                break;
            }
        }
    }

    let lcpos = match level_container_pos {
        Some(pos) => pos,
        None => return Vec::new(),
    };

    let slots_pat = b"Slots\x00";
    let mut level_slots_pos = None;
    for i in lcpos..std::cmp::min(lcpos + 15000, level_bytes.len()) {
        if i + slots_pat.len() <= level_bytes.len()
            && &level_bytes[i..i + slots_pat.len()] == slots_pat
        {
            level_slots_pos = Some(i - 4);
            break;
        }
    }

    let lspos = match level_slots_pos {
        Some(pos) => pos,
        None => return Vec::new(),
    };

    let mut offset = lspos;
    let _slots_str = read_string_at(level_bytes, &mut offset);
    let _type_str = read_string_at(level_bytes, &mut offset);
    offset += 8;
    let _item_type_str = read_string_at(level_bytes, &mut offset);
    offset += 1;

    if offset + 4 > level_bytes.len() {
        return Vec::new();
    }
    let count = u32::from_le_bytes([
        level_bytes[offset],
        level_bytes[offset + 1],
        level_bytes[offset + 2],
        level_bytes[offset + 3],
    ]);
    offset += 4;

    let _prop_name = read_string_at(level_bytes, &mut offset);
    let _prop_type = read_string_at(level_bytes, &mut offset);
    offset += 8;
    let _type_name = read_string_at(level_bytes, &mut offset);
    offset += 16 + 1;

    let mut items = Vec::new();
    for _ in 0..count {
        loop {
            let prop_name = read_string_at(level_bytes, &mut offset);
            if prop_name.is_empty() || prop_name == "None" {
                break;
            }
            let prop_type = read_string_at(level_bytes, &mut offset);
            let size = i64::from_le_bytes([
                level_bytes[offset],
                level_bytes[offset + 1],
                level_bytes[offset + 2],
                level_bytes[offset + 3],
                level_bytes[offset + 4],
                level_bytes[offset + 5],
                level_bytes[offset + 6],
                level_bytes[offset + 7],
            ]) as usize;
            offset += 8;

            if prop_name == "RawData" {
                let _array_item_type = read_string_at(level_bytes, &mut offset);
                offset += 1;

                let val_offset = offset;
                offset += size;

                let b_count = u32::from_le_bytes([
                    level_bytes[val_offset],
                    level_bytes[val_offset + 1],
                    level_bytes[val_offset + 2],
                    level_bytes[val_offset + 3],
                ]) as usize;

                if val_offset + 4 + b_count <= level_bytes.len() {
                    let raw_bytes = &level_bytes[val_offset + 4..val_offset + 4 + b_count];
                    let slot_idx = u32::from_le_bytes([
                        raw_bytes[0],
                        raw_bytes[1],
                        raw_bytes[2],
                        raw_bytes[3],
                    ]);
                    let stack_count = u32::from_le_bytes([
                        raw_bytes[4],
                        raw_bytes[5],
                        raw_bytes[6],
                        raw_bytes[7],
                    ]);
                    let id_len = u32::from_le_bytes([
                        raw_bytes[8],
                        raw_bytes[9],
                        raw_bytes[10],
                        raw_bytes[11],
                    ]) as usize;

                    if id_len > 1 && 12 + id_len - 1 <= raw_bytes.len() {
                        let item_id =
                            String::from_utf8_lossy(&raw_bytes[12..12 + id_len - 1]).into_owned();
                        if !item_id.is_empty() && stack_count > 0 {
                            items.push(InventoryItem {
                                slot_index: slot_idx,
                                item_id,
                                count: stack_count,
                            });
                        }
                    }
                }
                break;
            } else {
                skip_property_at(level_bytes, &mut offset, &prop_type, size);
            }
        }
    }

    items
}

pub fn find_pattern(bytes: &[u8], pattern: &[u8], start: usize) -> Option<usize> {
    if start + pattern.len() > bytes.len() {
        return None;
    }
    for i in start..=bytes.len() - pattern.len() {
        if &bytes[i..i + pattern.len()] == pattern {
            return Some(i);
        }
    }
    None
}

pub fn extract_any_float_prop(bytes: &[u8], prop_name: &[u8]) -> f64 {
    let mut pos = None;
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            pos = Some(i);
            break;
        }
    }
    let pos = match pos {
        Some(p) => p,
        None => return 0.0,
    };
    let mut offset = pos + prop_name.len();
    let t = read_string_at(bytes, &mut offset);
    if t == "FloatProperty" {
        offset += 8;
        offset += 1;
        if offset + 4 <= bytes.len() {
            return f32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as f64;
        }
    } else if t == "DoubleProperty" {
        offset += 8;
        offset += 1;
        if offset + 8 <= bytes.len() {
            return f64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
        }
    }
    0.0
}

pub fn extract_vector_coords_float(bytes: &[u8], offset: usize) -> (f64, f64, f64) {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    let window_end = (offset + 500).min(bytes.len());
    let window = &bytes[offset..window_end];

    if let Some(pos) = find_pattern(window, b"X\x00", 0) {
        x = extract_any_float_prop(&window[pos..], b"X\x00");
    }
    if let Some(pos) = find_pattern(window, b"Y\x00", 0) {
        y = extract_any_float_prop(&window[pos..], b"Y\x00");
    }
    if let Some(pos) = find_pattern(window, b"Z\x00", 0) {
        z = extract_any_float_prop(&window[pos..], b"Z\x00");
    }

    (x, y, z)
}

pub fn scan_base_camps(bytes: &[u8]) -> Vec<BaseCampSummary> {
    let mut camps = Vec::new();
    let pattern = b"BaseCampSaveData\x00";
    let mut last_pos = 0;
    while let Some(pos) = find_pattern(bytes, pattern, last_pos) {
        last_pos = pos + pattern.len();
        let window_end = (pos + 4000).min(bytes.len());
        let window = &bytes[pos..window_end];

        let mut level = 1;
        if let Some(lvl_pos) = find_pattern(window, b"BaseCampLevel\x00", 0) {
            level = extract_int_prop(&window[lvl_pos..], b"BaseCampLevel\x00") as u32;
            if level == 0 {
                level = 1;
            }
        }

        let mut group_id = String::new();
        if let Some(grp_pos) = find_pattern(window, b"GroupIdOfUser\x00", 0) {
            if let Some(guid) = extract_guid_prop(&window[grp_pos..], b"GroupIdOfUser\x00") {
                group_id = guid;
            }
        } else if let Some(grp_pos) = find_pattern(window, b"OwnerGroupId\x00", 0) {
            if let Some(guid) = extract_guid_prop(&window[grp_pos..], b"OwnerGroupId\x00") {
                group_id = guid;
            }
        }

        let mut coords = (0.0, 0.0, 0.0);
        if let Some(loc_pos) = find_pattern(window, b"Location\x00", 0) {
            coords = extract_vector_coords_float(window, loc_pos);
        }

        let base_camp_id = if pos >= 16 {
            let mut g_bytes = [0u8; 16];
            g_bytes.copy_from_slice(&bytes[pos - 16..pos]);
            format_guid(&g_bytes)
        } else {
            "00000000-0000-0000-0000-000000000000".to_string()
        };

        camps.push(BaseCampSummary {
            base_camp_id,
            group_id,
            level,
            coordinates: coords,
        });
    }
    camps
}

pub fn scan_guilds(bytes: &[u8]) -> Vec<GuildSummary> {
    let mut guilds = Vec::new();
    let pattern = b"EPalGroupType::Guild\x00";
    let mut last_pos = 0;
    while let Some(pos) = find_pattern(bytes, pattern, last_pos) {
        last_pos = pos + pattern.len();

        let window_start = pos.saturating_sub(500);
        let window_end = (pos + 4000).min(bytes.len());
        let window = &bytes[window_start..window_end];

        let mut guild_name = String::new();
        for name_pat in &[
            &b"guild_name\x00"[..],
            &b"GuildName\x00"[..],
            &b"GroupName\x00"[..],
            &b"group_name\x00"[..],
        ] {
            if let Some(name_pos) = find_pattern(window, name_pat, 0) {
                let name_val = extract_string_prop(&window[name_pos..], name_pat);
                if !name_val.is_empty() {
                    guild_name = name_val;
                    break;
                }
            }
        }
        if guild_name.is_empty() {
            guild_name = "Default Guild".to_string();
        }

        let mut admin_player_uid = String::new();
        for admin_pat in &[
            &b"admin_player_uid\x00"[..],
            &b"AdminPlayerUId\x00"[..],
            &b"OwnerPlayerUId\x00"[..],
        ] {
            if let Some(admin_pos) = find_pattern(window, admin_pat, 0) {
                if let Some(guid) = extract_guid_prop(&window[admin_pos..], admin_pat) {
                    admin_player_uid = guid;
                    break;
                }
            }
        }

        let mut members = std::collections::BTreeSet::new();
        let mut search_pos = 0;
        while let Some(member_pos) = find_pattern(window, b"PlayerUId\x00", search_pos) {
            search_pos = member_pos + 10;
            if let Some(guid) = extract_guid_prop(&window[member_pos..], b"PlayerUId\x00") {
                if guid != "00000000-0000-0000-0000-000000000000" {
                    members.insert(guid);
                }
            }
        }
        search_pos = 0;
        while let Some(member_pos) = find_pattern(window, b"player_uid\x00", search_pos) {
            search_pos = member_pos + 10;
            if let Some(guid) = extract_guid_prop(&window[member_pos..], b"player_uid\x00") {
                if guid != "00000000-0000-0000-0000-000000000000" {
                    members.insert(guid);
                }
            }
        }

        let mut guild_id = "00000000-0000-0000-0000-000000000000".to_string();
        if let Some(id_pos) = find_pattern(window, b"GroupId\x00", 0) {
            if let Some(guid) = extract_guid_prop(&window[id_pos..], b"GroupId\x00") {
                guild_id = guid;
            }
        }

        guilds.push(GuildSummary {
            guild_id,
            guild_name,
            admin_player_uid,
            members: members.into_iter().collect(),
        });
    }
    guilds
}

pub fn extract_vector_coords(bytes: &[u8], offset: &mut usize) -> (i32, i32, i32) {
    if *offset + 24 <= bytes.len() {
        let x = f64::from_le_bytes([
            bytes[*offset],
            bytes[*offset + 1],
            bytes[*offset + 2],
            bytes[*offset + 3],
            bytes[*offset + 4],
            bytes[*offset + 5],
            bytes[*offset + 6],
            bytes[*offset + 7],
        ]);
        let y = f64::from_le_bytes([
            bytes[*offset + 8],
            bytes[*offset + 9],
            bytes[*offset + 10],
            bytes[*offset + 11],
            bytes[*offset + 12],
            bytes[*offset + 13],
            bytes[*offset + 14],
            bytes[*offset + 15],
        ]);
        let z = f64::from_le_bytes([
            bytes[*offset + 16],
            bytes[*offset + 17],
            bytes[*offset + 18],
            bytes[*offset + 19],
            bytes[*offset + 20],
            bytes[*offset + 21],
            bytes[*offset + 22],
            bytes[*offset + 23],
        ]);
        *offset += 24;
        (x as i32, y as i32, z as i32)
    } else {
        (0, 0, 0)
    }
}

#[allow(clippy::type_complexity)]
pub fn find_chest_containers(level_bytes: &[u8]) -> Vec<([u8; 16], String, (i32, i32, i32))> {
    let mut results = Vec::new();
    let pattern = b"EPalMapObjectConcreteModelModuleType::ItemContainer\x00";
    let raw_data_pat = b"RawData\x00";

    let mut pos = 0;
    while let Some(match_idx) = level_bytes[pos..]
        .windows(pattern.len())
        .position(|w| w == pattern)
    {
        let abs_idx = pos + match_idx;
        pos = abs_idx + pattern.len();

        let search_end = std::cmp::min(abs_idx + 250, level_bytes.len());
        if let Some(rd_offset) = level_bytes[abs_idx..search_end]
            .windows(raw_data_pat.len())
            .position(|w| w == raw_data_pat)
        {
            let mut offset = abs_idx + rd_offset + raw_data_pat.len();
            let type_name = read_string_at(level_bytes, &mut offset);
            if type_name == "ArrayProperty" {
                offset += 8;
                let elem_type = read_string_at(level_bytes, &mut offset);
                offset += 1;
                if elem_type == "ByteProperty" && offset + 4 + 16 <= level_bytes.len() {
                    offset += 4;
                    let mut guid = [0u8; 16];
                    guid.copy_from_slice(&level_bytes[offset..offset + 16]);

                    let search_start = abs_idx.saturating_sub(2000);
                    let mut chest_type = "Chest/Storage".to_string();
                    let map_obj_pat = b"MapObjectId\x00";
                    if let Some(mo_offset) = level_bytes[search_start..abs_idx]
                        .windows(map_obj_pat.len())
                        .position(|w| w == map_obj_pat)
                    {
                        let mut temp_off = search_start + mo_offset + map_obj_pat.len();
                        let t = read_string_at(level_bytes, &mut temp_off);
                        if t == "StrProperty" || t == "NameProperty" {
                            temp_off += 8;
                            temp_off += 1;
                            chest_type = read_string_at(level_bytes, &mut temp_off);
                        }
                    }

                    let mut coords = (0, 0, 0);
                    let trans_pat = b"translation\x00";
                    if let Some(t_offset) = level_bytes[search_start..abs_idx + 1000]
                        .windows(trans_pat.len())
                        .position(|w| w == trans_pat)
                    {
                        let mut temp_off = search_start + t_offset + trans_pat.len();
                        let t = read_string_at(level_bytes, &mut temp_off);
                        if t == "StructProperty" {
                            temp_off += 8;
                            let st_type = read_string_at(level_bytes, &mut temp_off);
                            if st_type == "Vector" {
                                temp_off += 16 + 1;
                                coords = extract_vector_coords(level_bytes, &mut temp_off);
                            }
                        }
                    }

                    results.push((guid, chest_type, coords));
                }
            }
        }
    }
    results
}

pub fn clean_seeds_in_bytes(bytes: &mut [u8]) -> Vec<(String, u32)> {
    let mut cleaned = Vec::new();
    let pattern = b"RawData\x00";
    let mut pos = 0;

    while let Some(match_idx) = bytes[pos..]
        .windows(pattern.len())
        .position(|w| w == pattern)
    {
        let abs_idx = pos + match_idx;
        pos = abs_idx + pattern.len();

        let mut offset = abs_idx + pattern.len();
        if offset >= bytes.len() {
            continue;
        }
        let type_name = read_string_at(bytes, &mut offset);
        if type_name == "ArrayProperty" {
            offset += 8;
            let elem_type = read_string_at(bytes, &mut offset);
            offset += 1;
            if elem_type == "ByteProperty" && offset + 4 <= bytes.len() {
                let count = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]) as usize;
                offset += 4;
                if offset + count <= bytes.len() && count >= 12 {
                    let raw_start = offset;
                    let stack_count = u32::from_le_bytes([
                        bytes[raw_start + 4],
                        bytes[raw_start + 5],
                        bytes[raw_start + 6],
                        bytes[raw_start + 7],
                    ]);
                    let id_len = u32::from_le_bytes([
                        bytes[raw_start + 8],
                        bytes[raw_start + 9],
                        bytes[raw_start + 10],
                        bytes[raw_start + 11],
                    ]) as usize;
                    if id_len > 0 && id_len < 100 && raw_start + 12 + id_len <= bytes.len() {
                        let item_id_bytes = &bytes[raw_start + 12..raw_start + 12 + id_len - 1];
                        if let Ok(item_id) = std::str::from_utf8(item_id_bytes) {
                            let item_lower = item_id.to_lowercase();
                            if item_lower.contains("seed") && stack_count > 0 {
                                cleaned.push((item_id.to_string(), stack_count));
                                bytes[raw_start + 4] = 0;
                                bytes[raw_start + 5] = 0;
                                bytes[raw_start + 6] = 0;
                                bytes[raw_start + 7] = 0;
                                bytes[raw_start + 8] = 0;
                                bytes[raw_start + 9] = 0;
                                bytes[raw_start + 10] = 0;
                                bytes[raw_start + 11] = 0;
                                for i in 0..id_len {
                                    bytes[raw_start + 12 + i] = 0;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    cleaned
}

pub fn compress_and_write_gvas(
    path: &Path,
    decompressed: &[u8],
    original_header: &[u8],
) -> Result<(), String> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    if original_header.len() < 12 {
        return Err("Original header too short".to_string());
    }

    let magic = &original_header[8..11];
    if magic != b"PlZ" {
        return Err("Saving only supports Zlib (PlZ) compressed saves currently".to_string());
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(decompressed).map_err(|e| e.to_string())?;
    let compressed_payload = encoder.finish().map_err(|e| e.to_string())?;

    let mut out_file = File::create(path).map_err(|e| e.to_string())?;
    let uncompressed_len = decompressed.len() as u32;
    out_file
        .write_all(&uncompressed_len.to_le_bytes())
        .map_err(|e| e.to_string())?;
    let compressed_len = compressed_payload.len() as u32;
    out_file
        .write_all(&compressed_len.to_le_bytes())
        .map_err(|e| e.to_string())?;
    out_file
        .write_all(&original_header[8..12])
        .map_err(|e| e.to_string())?;
    out_file
        .write_all(&compressed_payload)
        .map_err(|e| e.to_string())?;

    Ok(())
}
