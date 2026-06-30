use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
mod i18n;
use flate2::read::ZlibDecoder;
use serde::Serialize;
use serde_json::Value;

type OodleDecompressFn = unsafe extern "C" fn(
    src: *const u8,
    src_len: usize,
    dst: *mut u8,
    dst_len: usize,
    fuzz: i32,
    crc: i32,
    verbose: i32,
    dst_base: *mut u8,
    e: usize,
    cb: *mut std::ffi::c_void,
    cb_data: *mut std::ffi::c_void,
    scratch: *mut std::ffi::c_void,
    scratch_size: usize,
    thread_phase: i32,
) -> i32;

fn decompress_oodle(data: &[u8], uncompressed_len: usize) -> Result<Vec<u8>, String> {
    unsafe {
        let mut search_paths = Vec::new();
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                search_paths.push(exe_dir.join("oo2core_9_win64.dll"));
                search_paths.push(exe_dir.join("..").join("..").join("oo2core_9_win64.dll"));
                search_paths.push(exe_dir.join("..").join("..").join("..").join("oo2core_9_win64.dll"));
            }
        }
        search_paths.push(PathBuf::from("oo2core_9_win64.dll"));

        let mut loaded_lib = None;
        for path in search_paths {
            if path.exists() {
                if let Ok(lib) = libloading::Library::new(&path) {
                    loaded_lib = Some(lib);
                    break;
                }
            }
        }

        let lib = match loaded_lib {
            Some(l) => l,
            None => libloading::Library::new("oo2core_9_win64.dll")
                .map_err(|e| format!("Failed to load oo2core_9_win64.dll: {}", e))?,
        };
            
        let oodle_decompress: libloading::Symbol<OodleDecompressFn> = lib.get(b"OodleLZ_Decompress")
            .map_err(|e| format!("Failed to find OodleLZ_Decompress symbol: {}", e))?;

        let mut decompressed = vec![0u8; uncompressed_len];
        
        let result = oodle_decompress(
            data.as_ptr(),
            data.len(),
            decompressed.as_mut_ptr(),
            uncompressed_len,
            0,
            0,
            0,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            3,
        );

        if result <= 0 {
            return Err(format!("Oodle decompression failed: {}", result));
        }

        Ok(decompressed)
    }
}

fn decompress_gvas(path: &Path) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| e.to_string())?;

    if data.len() < 12 {
        return Err("File too short".to_string());
    }

    let uncompressed_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let magic = &data[8..11];

    if magic == b"PlZ" {
        let mut decoder1 = ZlibDecoder::new(&data[12..]);
        let mut intermediate = Vec::new();
        if decoder1.read_to_end(&mut intermediate).is_err() {
            let mut decoder = ZlibDecoder::new(&data[12..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| e.to_string())?;
            return Ok(decompressed);
        }

        let mut decoder2 = ZlibDecoder::new(&intermediate[..]);
        let mut decompressed = Vec::with_capacity(uncompressed_len);
        if decoder2.read_to_end(&mut decompressed).is_ok() {
            Ok(decompressed)
        } else {
            Ok(intermediate)
        }
    } else if magic == b"PlM" {
        decompress_oodle(&data[12..], uncompressed_len)
    } else {
        Err("Unsupported magic".to_string())
    }
}

fn format_guid(bytes: &[u8; 16]) -> String {
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

fn read_string_at(bytes: &[u8], offset: &mut usize) -> String {
    if *offset + 4 > bytes.len() {
        return String::new();
    }
    let len = i32::from_le_bytes([bytes[*offset], bytes[*offset + 1], bytes[*offset + 2], bytes[*offset + 3]]);
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
        let chars: Vec<u16> = char_bytes.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let mut s = String::from_utf16_lossy(&chars);
        if s.ends_with('\0') {
            s.pop();
        }
        s
    }
}

// --- High-Performance Signature Search and Scanning Functions ---

fn extract_guid_prop(bytes: &[u8], prop_name: &[u8]) -> Option<String> {
    extract_guid_bytes_prop(bytes, prop_name).map(|g| format_guid(&g))
}

fn extract_guid_bytes_prop(bytes: &[u8], prop_name: &[u8]) -> Option<[u8; 16]> {
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
    let gpos = guid_pos?;
    if gpos + 38 <= bytes.len() {
        let mut g_bytes = [0u8; 16];
        g_bytes.copy_from_slice(&bytes[gpos + 22..gpos + 38]);
        Some(g_bytes)
    } else {
        None
    }
}

fn extract_string_prop(bytes: &[u8], prop_name: &[u8]) -> String {
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

fn extract_int_prop(bytes: &[u8], prop_name: &[u8]) -> i32 {
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
        i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]])
    } else {
        0
    }
}

fn extract_int64_prop(bytes: &[u8], prop_name: &[u8]) -> u64 {
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
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ])
    } else {
        0
    }
}

fn extract_byte_prop(bytes: &[u8], prop_name: &[u8]) -> u32 {
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

fn extract_float_prop(bytes: &[u8], prop_name: &[u8]) -> f64 {
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
        f32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as f64
    } else {
        0.0
    }
}

fn extract_fixed_point_prop(bytes: &[u8], prop_name: &[u8]) -> f64 {
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
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        v as f64 / 1000.0
    } else {
        0.0
    }
}

fn extract_array_strings(bytes: &[u8], prop_name: &[u8]) -> Vec<String> {
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
    if t != "ArrayProperty" { return results; }
    offset += 8; // skip size
    let elem_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator
    if offset + 4 > bytes.len() { return results; }
    let count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
    offset += 4;
    
    if elem_type == "StructProperty" {
        // Quest array element structure
        let _st_name = read_string_at(bytes, &mut offset);
        let _st_type = read_string_at(bytes, &mut offset);
        offset += 8; // skip size
        let _st_typename = read_string_at(bytes, &mut offset);
        offset += 16 + 1; // GUID + separator
        
        for _ in 0..count {
            if offset >= bytes.len() { break; }
            // Inside OrderedQuestArray, each struct contains QuestName (StrProperty)
            let q_name_pat = b"QuestName\x00";
            let mut q_pos = None;
            for i in offset..std::cmp::min(offset + 150, bytes.len()) {
                if i + q_name_pat.len() <= bytes.len() && &bytes[i..i + q_name_pat.len()] == q_name_pat {
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
            // Skip to end of struct property
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
                        bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                        bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
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
            if offset >= bytes.len() { break; }
            let s = read_string_at(bytes, &mut offset);
            if !s.is_empty() {
                results.push(s);
            }
        }
    }
    results
}

fn extract_map_keys(bytes: &[u8], map_name: &[u8]) -> Vec<String> {
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
    if t != "MapProperty" { return results; }
    offset += 8; // skip size
    let _key_type = read_string_at(bytes, &mut offset);
    let val_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator
    
    if offset + 8 > bytes.len() { return results; }
    let _keys_to_delete = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
    offset += 4;
    let count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
    offset += 4;
    
    for _ in 0..count {
        if offset >= bytes.len() { break; }
        let key = read_string_at(bytes, &mut offset);
        if key.is_empty() { break; }
        results.push(key);
        // Skip value depending on type
        if val_type == "BoolProperty" {
            offset += 1;
        } else if val_type == "IntProperty" {
            offset += 4;
        } else if val_type == "StructProperty" {
            // FastTravelPointUnlockFlag struct value is 16 bytes Guid
            offset += 16;
        } else {
            offset += 4;
        }
    }
    results
}

fn extract_map_counts(bytes: &[u8], map_name: &[u8]) -> HashMap<String, u32> {
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
    if t != "MapProperty" { return results; }
    offset += 8; // skip size
    let _key_type = read_string_at(bytes, &mut offset);
    let val_type = read_string_at(bytes, &mut offset);
    offset += 1; // separator
    
    if offset + 8 > bytes.len() { return results; }
    let _keys_to_delete = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
    offset += 4;
    let count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
    offset += 4;
    
    for _ in 0..count {
        if offset >= bytes.len() { break; }
        let key = read_string_at(bytes, &mut offset);
        if key.is_empty() { break; }
        let mut val = 0;
        if val_type == "IntProperty" {
            if offset + 4 <= bytes.len() {
                val = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
                offset += 4;
            }
        } else {
            offset += 4;
        }
        results.insert(key, val);
    }
    results
}

// --- End of Signature Search and Scanning Functions ---

struct CharacterEntry {
    player_uid: String,
    instance_id: String,
    raw_data: Vec<u8>,
}

fn scan_character_save_parameters(level_bytes: &[u8]) -> Vec<CharacterEntry> {
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
    if prop_type != "MapProperty" { return entries; }
    offset += 8; // skip size
    let _key_type = read_string_at(level_bytes, &mut offset);
    let _val_type = read_string_at(level_bytes, &mut offset);
    offset += 1; // separator

    if offset + 8 > level_bytes.len() { return entries; }
    let _keys_to_delete = u32::from_le_bytes([level_bytes[offset], level_bytes[offset+1], level_bytes[offset+2], level_bytes[offset+3]]);
    offset += 4;
    let count = u32::from_le_bytes([level_bytes[offset], level_bytes[offset+1], level_bytes[offset+2], level_bytes[offset+3]]) as usize;
    offset += 4;

    for _ in 0..count {
        if offset >= level_bytes.len() { break; }
        
        // Parse key
        let mut player_uid = String::new();
        let mut instance_id = String::new();
        
        loop {
            let p_name = read_string_at(level_bytes, &mut offset);
            if p_name.is_empty() || p_name == "None" {
                break;
            }
            let p_type = read_string_at(level_bytes, &mut offset);
            let size = i64::from_le_bytes([
                level_bytes[offset], level_bytes[offset+1], level_bytes[offset+2], level_bytes[offset+3],
                level_bytes[offset+4], level_bytes[offset+5], level_bytes[offset+6], level_bytes[offset+7]
            ]) as usize;
            offset += 8;
            
            let mut struct_type = String::new();
            if p_type == "StructProperty" {
                struct_type = read_string_at(level_bytes, &mut offset);
                offset += 17; // GUID + separator
            } else {
                offset += 1;
            }
            
            let val_offset = offset;
            offset += size;
            
            if p_name == "PlayerUId" && struct_type == "Guid" {
                let mut g_bytes = [0u8; 16];
                g_bytes.copy_from_slice(&level_bytes[val_offset..val_offset+16]);
                player_uid = format_guid(&g_bytes);
            } else if p_name == "InstanceId" && struct_type == "Guid" {
                let mut g_bytes = [0u8; 16];
                g_bytes.copy_from_slice(&level_bytes[val_offset..val_offset+16]);
                instance_id = format_guid(&g_bytes);
            }
        }
        
        // Parse value (contains RawData)
        let mut raw_data = Vec::new();
        loop {
            let p_name = read_string_at(level_bytes, &mut offset);
            if p_name.is_empty() || p_name == "None" {
                break;
            }
            let p_type = read_string_at(level_bytes, &mut offset);
            let size = i64::from_le_bytes([
                level_bytes[offset], level_bytes[offset+1], level_bytes[offset+2], level_bytes[offset+3],
                level_bytes[offset+4], level_bytes[offset+5], level_bytes[offset+6], level_bytes[offset+7]
            ]) as usize;
            offset += 8;
            
            if p_type == "ArrayProperty" {
                let array_type = read_string_at(level_bytes, &mut offset);
                offset += 1; // separator
                
                let val_offset = offset;
                offset += size;
                
                if p_name == "RawData" && array_type == "ByteProperty" {
                    let b_count = u32::from_le_bytes([
                        level_bytes[val_offset], level_bytes[val_offset+1], level_bytes[val_offset+2], level_bytes[val_offset+3]
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
            entries.push(CharacterEntry { player_uid, instance_id, raw_data });
        }
    }
    entries
}

#[derive(Serialize, Debug)]
struct InventoryItem {
    slot_index: u32,
    item_id: String,
    count: u32,
}

#[derive(Serialize, Debug)]
struct PalSummary {
    character_id: String,
    gender: String,
    level: u32,
    exp: u64,
    hp: f64,
    max_hp: f64,
    satiety: f64,
    physical_health: String,
    friendship: u32,
    talents: HashMap<String, u32>,
    passive_skills: Vec<String>,
    slot_index: u32,
}

#[derive(Serialize, Debug)]
struct PlayerSummary {
    player_uid: String,
    instance_id: String,
    nickname: String,
    level: u32,
    exp: u64,
    hp: f64,
    max_hp: f64,
    full_stomach: f64,
    physical_health: String,
    technology_points: u32,
    customization: HashMap<String, Value>,
    unlocked_technologies: Vec<String>,
    active_quest: String,
    completed_quests: Vec<String>,
    fast_travel_points: Vec<String>,
    relics_found: u32,
    notes_found: Vec<String>,
    npc_talk_counts: HashMap<String, u32>,
    common_inventory: Vec<InventoryItem>,
    weapons: Vec<InventoryItem>,
    armor: Vec<InventoryItem>,
    active_pals: Vec<PalSummary>,
}

#[derive(Serialize, Debug)]
struct OutputJson {
    status: String,
    world_path: String,
    players: Vec<PlayerSummary>,
}

fn skip_property_at(bytes: &[u8], offset: &mut usize, type_name: &str, size: usize) {
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
}fn has_prop(bytes: &[u8], prop_name: &[u8]) -> bool {
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            return true;
        }
    }
    false
}

fn parse_container_items(level_bytes: &[u8], container_guid_le: &[u8; 16]) -> Vec<InventoryItem> {
    let mut level_container_pos = None;
    for i in 0..level_bytes.len() - 16 {
        if &level_bytes[i..i + 16] == container_guid_le {
            let check_end = std::cmp::min(i + 40, level_bytes.len());
            if level_bytes[i + 16..check_end].windows(10).any(|w| w == b"BelongInfo") {
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
        if i + slots_pat.len() <= level_bytes.len() && &level_bytes[i..i + slots_pat.len()] == slots_pat {
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
    offset += 8; // skip size
    let _item_type_str = read_string_at(level_bytes, &mut offset);
    offset += 1; // separator
    
    if offset + 4 > level_bytes.len() { return Vec::new(); }
    let count = u32::from_le_bytes([level_bytes[offset], level_bytes[offset+1], level_bytes[offset+2], level_bytes[offset+3]]);
    offset += 4;

    let _prop_name = read_string_at(level_bytes, &mut offset);
    let _prop_type = read_string_at(level_bytes, &mut offset);
    offset += 8; // skip size
    let _type_name = read_string_at(level_bytes, &mut offset);
    offset += 16 + 1; // GUID + separator

    let mut items = Vec::new();
    for _ in 0..count {
        loop {
            let prop_name = read_string_at(level_bytes, &mut offset);
            if prop_name.is_empty() || prop_name == "None" {
                break;
            }
            let prop_type = read_string_at(level_bytes, &mut offset);
            let size = i64::from_le_bytes([
                level_bytes[offset], level_bytes[offset+1], level_bytes[offset+2], level_bytes[offset+3],
                level_bytes[offset+4], level_bytes[offset+5], level_bytes[offset+6], level_bytes[offset+7]
            ]) as usize;
            offset += 8;

            if prop_name == "RawData" {
                let _array_item_type = read_string_at(level_bytes, &mut offset);
                offset += 1; // separator
                
                let val_offset = offset;
                offset += size;
                
                let b_count = u32::from_le_bytes([
                    level_bytes[val_offset], level_bytes[val_offset+1], level_bytes[val_offset+2], level_bytes[val_offset+3]
                ]) as usize;
                
                if val_offset + 4 + b_count <= level_bytes.len() {
                    let raw_bytes = &level_bytes[val_offset + 4..val_offset + 4 + b_count];
                    let slot_idx = u32::from_le_bytes([raw_bytes[0], raw_bytes[1], raw_bytes[2], raw_bytes[3]]);
                    let stack_count = u32::from_le_bytes([raw_bytes[4], raw_bytes[5], raw_bytes[6], raw_bytes[7]]);
                    let id_len = u32::from_le_bytes([raw_bytes[8], raw_bytes[9], raw_bytes[10], raw_bytes[11]]) as usize;
                    
                    if id_len > 1 && 12 + id_len - 1 <= raw_bytes.len() {
                        let item_id = String::from_utf8_lossy(&raw_bytes[12..12 + id_len - 1]).into_owned();
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

fn detect_save_dir() -> Option<PathBuf> {
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let path = std::path::Path::new(&local_appdata)
            .join("Pal")
            .join("Saved")
            .join("SaveGames");
        if path.exists() {
            let mut valid_worlds = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let user_path = entry.path();
                    if user_path.is_dir() {
                        if let Ok(sub_entries) = std::fs::read_dir(&user_path) {
                            for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                                let world_path = sub_entry.path();
                                if world_path.is_dir() && world_path.file_name().unwrap() != "Cloud" {
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
            if !valid_worlds.is_empty() {
                valid_worlds.sort_by(|a, b| b.1.cmp(&a.1));
                return Some(valid_worlds[0].0.clone());
            }
        }
    }
    None
}

fn main() {
    i18n::init(i18n::detect_system_language());

    let args_list: Vec<String> = std::env::args().skip(1).collect();
    let is_json = args_list.iter().any(|arg| arg == "--json");
    let world_path_arg = args_list.iter().find(|arg| !arg.starts_with("-")).cloned();

    let world_path = match world_path_arg {
        Some(ref p) => PathBuf::from(p),
        None => match detect_save_dir() {
            Some(p) => p,
            None => {
                let err_json = serde_json::json!({
                    "status": "error",
                    "message": i18n::t("error_detect_save")
                });
                println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                std::process::exit(1);
            }
        }
    };

    let level_sav = world_path.join("Level.sav");
    if !level_sav.exists() {
        let err_json = serde_json::json!({
            "status": "error",
            "message": format!("Level.sav not found at {}", level_sav.display())
        });
        println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
        std::process::exit(1);
    }

    let level_bytes = match decompress_gvas(&level_sav) {
        Ok(b) => b,
        Err(e) => {
            let err_json = serde_json::json!({
                "status": "error",
                "message": format!("Failed to decompress Level.sav: {}", e)
            });
            println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
            std::process::exit(1);
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

                    if let Ok(player_bytes) = decompress_gvas(&path) {
                        // Extract player stats from character entries
                        let player_char_entry = characters.iter().find(|c| c.player_uid == player_uid_str);
                        let mut level = 1;
                        let mut exp = 0;
                        let mut hp = 0.0;
                        let mut max_hp = 0.0;
                        let mut full_stomach = 0.0;
                        let mut physical_health = "EPalStatusPhysicalHealthType::Normal".to_string();
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
                            if level == 0 { level = 1; }
                            exp = extract_int64_prop(&char_entry.raw_data, b"Exp\x00");
                            hp = extract_fixed_point_prop(&char_entry.raw_data, b"Hp\x00");
                            max_hp = extract_fixed_point_prop(&char_entry.raw_data, b"MaxHp\x00");
                            full_stomach = extract_float_prop(&char_entry.raw_data, b"FullStomach\x00");
                            let ph = extract_string_prop(&char_entry.raw_data, b"PhysicalHealth\x00");
                            if !ph.is_empty() {
                                physical_health = ph;
                            }
                        }

                        // Extract Guids using signature search
                        let otomo_bytes = extract_guid_bytes_prop(&player_bytes, b"OtomoCharacterContainerId\x00");
                        let common_bytes = extract_guid_bytes_prop(&player_bytes, b"CommonContainerId\x00");
                        let weapon_bytes = extract_guid_bytes_prop(&player_bytes, b"WeaponLoadOutContainerId\x00");
                        let armor_bytes = extract_guid_bytes_prop(&player_bytes, b"PlayerEquipArmorContainerId\x00");
                        let otomo_id = otomo_bytes.map(|b| format_guid(&b)).unwrap_or_default();

                        // Parse inventory items
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

                        // Extract active Pals
                        let mut active_pals = Vec::new();
                        let player_instance_id = player_char_entry.map(|c| c.instance_id.clone()).unwrap_or_default();
                        for char_entry in &characters {
                            if !player_instance_id.is_empty() && char_entry.instance_id != player_instance_id {
                                let owner_uid = extract_guid_prop(&char_entry.raw_data, b"OwnerPlayerUId\x00").unwrap_or_default();
                                if owner_uid == player_uid_str {
                                    let pal_container_id = extract_guid_prop(&char_entry.raw_data, b"ContainerId\x00").unwrap_or_default();
                                    if !otomo_id.is_empty() && pal_container_id == otomo_id {
                                        let char_id = extract_string_prop(&char_entry.raw_data, b"CharacterID\x00");
                                        let pal_gender = extract_string_prop(&char_entry.raw_data, b"Gender\x00");
                                        let pal_level = extract_byte_prop(&char_entry.raw_data, b"Level\x00");
                                        let pal_exp = extract_int64_prop(&char_entry.raw_data, b"Exp\x00");
                                        let pal_hp = extract_fixed_point_prop(&char_entry.raw_data, b"Hp\x00");
                                        let pal_max_hp = extract_fixed_point_prop(&char_entry.raw_data, b"MaxHp\x00");
                                        let pal_satiety = extract_float_prop(&char_entry.raw_data, b"FullStomach\x00");
                                        let pal_ph = extract_string_prop(&char_entry.raw_data, b"PhysicalHealth\x00");
                                        let friendship = extract_int_prop(&char_entry.raw_data, b"FriendshipPoint\x00") as u32;
                                        let slot_index = extract_int_prop(&char_entry.raw_data, b"SlotIndex\x00") as u32;

                                        let mut talents = HashMap::new();
                                        if has_prop(&char_entry.raw_data, b"Talent_HP\x00") {
                                            talents.insert("HP".to_string(), extract_int_prop(&char_entry.raw_data, b"Talent_HP\x00") as u32);
                                        }
                                        if has_prop(&char_entry.raw_data, b"Talent_Shot\x00") {
                                            talents.insert("Shot".to_string(), extract_int_prop(&char_entry.raw_data, b"Talent_Shot\x00") as u32);
                                        }
                                        if has_prop(&char_entry.raw_data, b"Talent_Defense\x00") {
                                            talents.insert("Defense".to_string(), extract_int_prop(&char_entry.raw_data, b"Talent_Defense\x00") as u32);
                                        }

                                        let passive_skills = extract_array_strings(&char_entry.raw_data, b"PassiveSkillList\x00");

                                        active_pals.push(PalSummary {
                                            character_id: if char_id.is_empty() { "Unknown".to_string() } else { char_id },
                                            gender: if pal_gender.is_empty() { "EPalGenderType::Male".to_string() } else { pal_gender },
                                            level: if pal_level == 0 { 1 } else { pal_level },
                                            exp: pal_exp,
                                            hp: pal_hp,
                                            max_hp: pal_max_hp,
                                            satiety: pal_satiety,
                                            physical_health: if pal_ph.is_empty() { "EPalStatusPhysicalHealthType::Normal".to_string() } else { pal_ph },
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

                        // Extract technology, quests, and exploration metrics via signature search
                        let technology_points = extract_int_prop(&player_bytes, b"TechnologyPoint\x00") as u32;
                        let unlocked_technologies = extract_array_strings(&player_bytes, b"UnlockedRecipeTechnologyNames\x00");
                        let active_quest = extract_string_prop(&player_bytes, b"QuestName\x00");
                        let completed_quests = extract_array_strings(&player_bytes, b"CompletedQuestArray\x00");

                        // RecordData metrics
                        let relics_found = extract_int_prop(&player_bytes, b"RelicPossessNum\x00") as u32;
                        let fast_travel_points = extract_map_keys(&player_bytes, b"FastTravelPointUnlockFlag\x00");
                        let notes_found = extract_map_keys(&player_bytes, b"NoteObtainForInstanceFlag\x00");
                        let npc_talk_counts = extract_map_counts(&player_bytes, b"NPCTalkCountMap\x00");

                        // Customizations
                        let mut customization = HashMap::new();
                        let b_mesh = extract_string_prop(&player_bytes, b"BodyMeshName\x00");
                        if !b_mesh.is_empty() { customization.insert("BodyMeshName".to_string(), Value::String(b_mesh)); }
                        let h_mesh = extract_string_prop(&player_bytes, b"HeadMeshName\x00");
                        if !h_mesh.is_empty() { customization.insert("HeadMeshName".to_string(), Value::String(h_mesh)); }
                        let hair_mesh = extract_string_prop(&player_bytes, b"HairMeshName\x00");
                        if !hair_mesh.is_empty() { customization.insert("HairMeshName".to_string(), Value::String(hair_mesh)); }
                        let voice_id = extract_int_prop(&player_bytes, b"VoiceID\x00");
                        if voice_id > 0 { customization.insert("VoiceID".to_string(), Value::Number(voice_id.into())); }

                        players.push(PlayerSummary {
                            player_uid: player_uid_str,
                            instance_id: player_char_entry.map(|c| c.instance_id.clone()).unwrap_or_default(),
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
                        });
                    }
                }
            }
        }
    }

    let output = OutputJson {
        status: "success".to_string(),
        world_path: world_path.to_string_lossy().into_owned(),
        players,
    };

    if is_json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        print_beautiful_report(&output);
    }
}

fn print_beautiful_report(output: &OutputJson) {
    let title = i18n::t("report_title");
    let border = "=".repeat(title.len() + 16);
    println!("{}", border);
    println!("        {}        ", title);
    println!("{}", border);
    println!("{}: {}", i18n::t("world_save_path"), output.world_path);
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
        println!("  * {:30} : {:.2} / 100", i18n::t("satiety_stomach"), player.full_stomach);
        println!("  * {:30} : {}", i18n::t("physical_status"), i18n::t(&player.physical_health));
        println!("  * {:30} : {}", i18n::t("tech_points_available"), player.technology_points);
        println!();

        println!("  [{}]", i18n::t("appearance_customization"));
        println!("  * {:30} : {}", i18n::t("body_type"), i18n::t(player.customization.get("BodyMeshName").and_then(|v| v.as_str()).unwrap_or("Unknown")));
        println!("  * {:30} : {}", i18n::t("head_model"), player.customization.get("HeadMeshName").and_then(|v| v.as_str()).unwrap_or("Unknown"));
        println!("  * {:30} : {}", i18n::t("hair_model"), player.customization.get("HairMeshName").and_then(|v| v.as_str()).unwrap_or("Unknown"));
        println!("  * {:30} : {}", i18n::t("voice_selection_id"), player.customization.get("VoiceID").and_then(|v| v.as_u64()).map(|v| v.to_string()).unwrap_or_else(|| "Unknown".to_string()));
        println!();

        println!("  [{}]", i18n::t("active_quest_progression"));
        println!("  * {:30} : {}", i18n::t("active_quest"), i18n::t(&player.active_quest));
        let completed_translated: Vec<String> = player.completed_quests.iter()
            .map(|q| i18n::t(q))
            .collect();
        println!("  * {:30} : {}", i18n::t("completed_quests"), completed_translated.join(", "));
        println!();

        println!("  [{}]", i18n::t("exploration_discovery"));
        println!("  * {:30} : {}", i18n::t("relics_found"), player.relics_found);
        println!("  * {:30} : {}", i18n::t("notes_read"), player.notes_found.join(", "));
        println!("  * {:30} : {} {}", i18n::t("fast_travel_unlocked"), player.fast_travel_points.len(), if player.fast_travel_points.len() == 1 { "point" } else { "points" });
        println!();

        println!("  [{} ({})]", i18n::t("active_team_pals"), player.active_pals.len());
        if player.active_pals.is_empty() {
            println!("    {}", i18n::t("no_pals_in_team"));
        } else {
            for (idx, pal) in player.active_pals.iter().enumerate() {
                let gender_str = if pal.gender.contains("Female") { "Female" } else { "Male" };
                println!("    {}. {} [{} {}] ({})", idx + 1, i18n::t(&pal.character_id), i18n::t("level"), pal.level, i18n::t(gender_str));
                println!("       - HP                 : {:.2} / {:.2}", pal.hp, pal.max_hp);
                println!("       - Satiety (Stomach)  : {:.2}", pal.satiety);
                println!("       - Status             : {}", i18n::t(&pal.physical_health));
                println!("       - Friendship         : {}", pal.friendship);
                
                let talents_str = format!(
                    "HP: {}, Atk: {}, Def: {}",
                    pal.talents.get("HP").unwrap_or(&0),
                    pal.talents.get("Shot").unwrap_or(&0),
                    pal.talents.get("Defense").unwrap_or(&0)
                );
                println!("       - {:18} : {}", i18n::t("talents"), talents_str);

                if !pal.passive_skills.is_empty() {
                    let passives_str = pal.passive_skills.iter()
                        .map(|p| i18n::t(p))
                        .collect::<Vec<String>>()
                        .join(", ");
                    println!("       - {:18} : {}", i18n::t("passive_skills"), passives_str);
                }
            }
        }
        println!();

        println!("  [{}]", i18n::t("equipped_weapons"));
        if player.weapons.is_empty() {
            println!("    {}", i18n::t("no_weapons_equipped"));
        } else {
            for item in &player.weapons {
                println!("    * Slot {}: {} (x{})", item.slot_index, item.item_id, item.count);
            }
        }
        println!();

        println!("  [{}]", i18n::t("equipped_armor"));
        if player.armor.is_empty() {
            println!("    {}", i18n::t("no_armor_equipped"));
        } else {
            for item in &player.armor {
                println!("    * Slot {}: {} (x{})", item.slot_index, item.item_id, item.count);
            }
        }
        println!();

        println!("  [{}]", i18n::t("backpack_inventory"));
        if player.common_inventory.is_empty() {
            println!("    {}", i18n::t("backpack_empty"));
        } else {
            for item in &player.common_inventory {
                println!("    * Slot {}: {} (x{})", item.slot_index, item.item_id, item.count);
            }
        }
        println!();
    }
    println!("{}", border);
}
