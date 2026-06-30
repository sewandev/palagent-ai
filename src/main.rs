use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
mod i18n;
use flate2::read::ZlibDecoder;
use serde::Serialize;
use serde_json::Value;

thread_local! {
    static OUTPUT_BUFFER: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

macro_rules! print {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        let captured = OUTPUT_BUFFER.with(|buf| {
            let mut b = buf.borrow_mut();
            if let Some(ref mut s) = *b {
                s.push_str(&msg);
                true
            } else {
                false
            }
        });
        if !captured {
            use std::io::Write;
            let _ = std::io::stdout().write_all(msg.as_bytes());
        }
    }};
}

macro_rules! println {
    () => {
        print!("\n")
    };
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        let captured = OUTPUT_BUFFER.with(|buf| {
            let mut b = buf.borrow_mut();
            if let Some(ref mut s) = *b {
                s.push_str(&msg);
                s.push('\n');
                true
            } else {
                false
            }
        });
        if !captured {
            use std::io::Write;
            let _ = std::io::stdout().write_all(msg.as_bytes());
            let _ = std::io::stdout().write_all(b"\n");
        }
    }};
}

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
                search_paths.push(
                    exe_dir
                        .join("..")
                        .join("..")
                        .join("..")
                        .join("oo2core_9_win64.dll"),
                );
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

        let oodle_decompress: libloading::Symbol<OodleDecompressFn> = lib
            .get(b"OodleLZ_Decompress")
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
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| e.to_string())?;
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
        // Quest array element structure
        let _st_name = read_string_at(bytes, &mut offset);
        let _st_type = read_string_at(bytes, &mut offset);
        offset += 8; // skip size
        let _st_typename = read_string_at(bytes, &mut offset);
        offset += 16 + 1; // GUID + separator

        for _ in 0..count {
            if offset >= bytes.len() {
                break;
            }
            // Inside OrderedQuestArray, each struct contains QuestName (StrProperty)
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
    if prop_type != "MapProperty" {
        return entries;
    }
    offset += 8; // skip size
    let _key_type = read_string_at(level_bytes, &mut offset);
    let _val_type = read_string_at(level_bytes, &mut offset);
    offset += 1; // separator

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
                offset += 17; // GUID + separator
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

        // Parse value (contains RawData)
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
                offset += 1; // separator

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
    palbox_pals: Vec<PalSummary>,
}

#[derive(Serialize, Debug, Clone)]
struct BaseCampSummary {
    base_camp_id: String,
    group_id: String,
    level: u32,
    coordinates: (f64, f64, f64),
}

#[derive(Serialize, Debug, Clone)]
struct GuildSummary {
    guild_id: String,
    guild_name: String,
    admin_player_uid: String,
    members: Vec<String>,
}

#[derive(Serialize, Debug)]
struct OutputJson {
    status: String,
    world_path: String,
    game_mode: String,
    players: Vec<PlayerSummary>,
    base_camps: Vec<BaseCampSummary>,
    guilds: Vec<GuildSummary>,
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
}

fn has_prop(bytes: &[u8], prop_name: &[u8]) -> bool {
    for i in 0..bytes.len() - prop_name.len() {
        if &bytes[i..i + prop_name.len()] == prop_name {
            return true;
        }
    }
    false
}

fn extract_bool_prop(bytes: &[u8], prop_name: &[u8]) -> bool {
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

fn extract_enum_prop(bytes: &[u8], prop_name: &[u8]) -> String {
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

fn parse_container_items(level_bytes: &[u8], container_guid_le: &[u8; 16]) -> Vec<InventoryItem> {
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
    offset += 8; // skip size
    let _item_type_str = read_string_at(level_bytes, &mut offset);
    offset += 1; // separator

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
                offset += 1; // separator

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

fn find_pattern(bytes: &[u8], pattern: &[u8], start: usize) -> Option<usize> {
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

fn extract_any_float_prop(bytes: &[u8], prop_name: &[u8]) -> f64 {
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
        offset += 8; // skip size
        offset += 1; // separator
        if offset + 4 <= bytes.len() {
            return f32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as f64;
        }
    } else if t == "DoubleProperty" {
        offset += 8; // skip size
        offset += 1; // separator
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

fn extract_vector_coords_float(bytes: &[u8], offset: usize) -> (f64, f64, f64) {
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

fn scan_base_camps(bytes: &[u8]) -> Vec<BaseCampSummary> {
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

fn scan_guilds(bytes: &[u8]) -> Vec<GuildSummary> {
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

fn get_all_detected_worlds() -> Vec<(PathBuf, std::time::SystemTime)> {
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

fn select_world_interactively(worlds: &[(PathBuf, std::time::SystemTime)]) -> PathBuf {
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
    use std::io::Write;
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

fn detect_game_mode(world_path: &Path) -> String {
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

static BREED_POWER: &[(&str, u32)] = &[
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

fn find_child_pal(power_a: u32, power_b: u32) -> (&'static str, u32) {
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

fn extract_vector_coords(bytes: &[u8], offset: &mut usize) -> (i32, i32, i32) {
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
fn find_chest_containers(level_bytes: &[u8]) -> Vec<([u8; 16], String, (i32, i32, i32))> {
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
                offset += 8; // skip size
                let elem_type = read_string_at(level_bytes, &mut offset);
                offset += 1; // separator
                if elem_type == "ByteProperty" && offset + 4 + 16 <= level_bytes.len() {
                    offset += 4; // skip count
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

fn clean_seeds_in_bytes(bytes: &mut [u8]) -> Vec<(String, u32)> {
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

fn compress_and_write_gvas(
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

fn run_time_command(world_path: &Path, is_json: bool) {
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

fn run_settings_command(world_path: &Path, is_json: bool) {
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

fn run_search_chest_command(world_path: &Path, query: &str, is_json: bool) {
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

fn run_breeding_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
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
            if char_id.is_empty() {
                continue;
            }
            let translated_name = i18n::t(&char_id);

            let has_power = BREED_POWER.iter().any(|&(name, _)| name == translated_name);
            if !has_power {
                continue;
            }

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

fn run_progress_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
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

    let relics = extract_map_keys(&bytes, b"RelicObtainForInstanceFlag\x00");
    let notes = extract_map_keys(&bytes, b"NoteObtainForInstanceFlag\x00");
    let fast_travels = extract_map_keys(&bytes, b"FastTravelPointUnlockFlag\x00");
    let areas = extract_map_keys(&bytes, b"FindAreaFlagMap\x00");
    let captures = extract_map_counts(&bytes, b"PalCaptureCount\x00");

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

fn run_clean_seeds_command(world_path: &Path, is_json: bool) {
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

fn run_monitor_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
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
            if has_prop(&char_entry.raw_data, b"SanityValue\x00") {
                san_val = extract_float_prop(&char_entry.raw_data, b"SanityValue\x00");
            }

            let mut satiety = 100.0;
            if has_prop(&char_entry.raw_data, b"FullStomach\x00") {
                satiety = extract_float_prop(&char_entry.raw_data, b"FullStomach\x00");
            }

            // Evaluate status
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
            if has_prop(&char_entry.raw_data, b"SanityValue\x00") {
                san_val = extract_float_prop(&char_entry.raw_data, b"SanityValue\x00");
            }

            let mut satiety = 100.0;
            if has_prop(&char_entry.raw_data, b"FullStomach\x00") {
                satiety = extract_float_prop(&char_entry.raw_data, b"FullStomach\x00");
            }

            let pal_name = i18n::t(&char_id);

            // Evaluate status
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

fn run_analyzer_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
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

            let iv_hp = if has_prop(&char_entry.raw_data, b"Talent_HP\x00") {
                extract_int_prop(&char_entry.raw_data, b"Talent_HP\x00")
            } else {
                0
            };

            let iv_atk = if has_prop(&char_entry.raw_data, b"Talent_Shot\x00") {
                extract_int_prop(&char_entry.raw_data, b"Talent_Shot\x00")
            } else {
                0
            };

            let iv_def = if has_prop(&char_entry.raw_data, b"Talent_Defense\x00") {
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
                "passive_skills_translated": passive_skills_translated
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

            let iv_hp = if has_prop(&char_entry.raw_data, b"Talent_HP\x00") {
                extract_int_prop(&char_entry.raw_data, b"Talent_HP\x00")
            } else {
                0
            };

            let iv_atk = if has_prop(&char_entry.raw_data, b"Talent_Shot\x00") {
                extract_int_prop(&char_entry.raw_data, b"Talent_Shot\x00")
            } else {
                0
            };

            let iv_def = if has_prop(&char_entry.raw_data, b"Talent_Defense\x00") {
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

fn run_full_command(world_path: &Path, is_json: bool, target_uid: Option<&str>) {
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
                                        if has_prop(&char_entry.raw_data, b"Talent_HP\x00") {
                                            talents.insert(
                                                "HP".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_HP\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if has_prop(&char_entry.raw_data, b"Talent_Shot\x00") {
                                            talents.insert(
                                                "Shot".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_Shot\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if has_prop(&char_entry.raw_data, b"Talent_Defense\x00") {
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
                                        if has_prop(&char_entry.raw_data, b"Talent_HP\x00") {
                                            talents.insert(
                                                "HP".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_HP\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if has_prop(&char_entry.raw_data, b"Talent_Shot\x00") {
                                            talents.insert(
                                                "Shot".to_string(),
                                                extract_int_prop(
                                                    &char_entry.raw_data,
                                                    b"Talent_Shot\x00",
                                                )
                                                    as u32,
                                            );
                                        }
                                        if has_prop(&char_entry.raw_data, b"Talent_Defense\x00") {
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
                        let fast_travel_points =
                            extract_map_keys(&player_bytes, b"FastTravelPointUnlockFlag\x00");
                        let notes_found =
                            extract_map_keys(&player_bytes, b"NoteObtainForInstanceFlag\x00");
                        let npc_talk_counts =
                            extract_map_counts(&player_bytes, b"NPCTalkCountMap\x00");

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

fn execute_command_captured(
    world_path: &Path,
    cmd: &str,
    is_json: bool,
    target_uid: Option<&str>,
) -> String {
    OUTPUT_BUFFER.with(|buf| {
        *buf.borrow_mut() = Some(String::new());
    });

    match cmd {
        "time" => run_time_command(world_path, is_json),
        "settings" => run_settings_command(world_path, is_json),
        "breeding" => run_breeding_command(world_path, is_json, target_uid),
        "progress" => run_progress_command(world_path, is_json, target_uid),
        "clean-seeds" => run_clean_seeds_command(world_path, is_json),
        "monitor" => run_monitor_command(world_path, is_json, target_uid),
        "analyzer" => run_analyzer_command(world_path, is_json, target_uid),
        c if c.starts_with("search-chest:") => {
            let query = &c["search-chest:".len()..];
            run_search_chest_command(world_path, query, is_json);
        }
        "full" => {
            run_full_command(world_path, is_json, target_uid);
        }
        _ => println!("Unknown command"),
    }

    OUTPUT_BUFFER.with(|buf| buf.borrow_mut().take().unwrap_or_default())
}

fn send_http_response(
    stream: &mut std::net::TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) {
    use std::io::Write;
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn handle_host_connection(mut stream: std::net::TcpStream, world_path: &Path, passcode: &str) {
    let mut buffer = [0u8; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };
    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let first_line = match request_str.lines().next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_http_response(&mut stream, "400 Bad Request", "text/plain", "Bad Request");
        return;
    }

    let path_query = parts[1];
    let (path, query) = if let Some(idx) = path_query.find('?') {
        (&path_query[..idx], &path_query[idx + 1..])
    } else {
        (path_query, "")
    };

    if path != "/api/command" {
        send_http_response(&mut stream, "404 Not Found", "text/plain", "Not Found");
        return;
    }

    let mut params = HashMap::new();
    for pair in query.split('&') {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 {
            params.insert(kv[0], kv[1]);
        }
    }

    let req_passcode = params.get("passcode").copied().unwrap_or("");
    if req_passcode != passcode {
        send_http_response(
            &mut stream,
            "403 Forbidden",
            "text/plain",
            "Access Denied: Invalid passcode.",
        );
        return;
    }

    let cmd = params.get("cmd").copied().unwrap_or("full");
    let is_json = params.get("is_json").copied().unwrap_or("false") == "true";
    let target_uid = params.get("uid").copied().map(|s| s.to_string());

    let result = execute_command_captured(world_path, cmd, is_json, target_uid.as_deref());
    send_http_response(&mut stream, "200 OK", "text/plain; charset=utf-8", &result);
}

fn start_host_server(world_path: PathBuf, port: u16, passcode: String) {
    let address = format!("0.0.0.0:{}", port);
    let listener = match std::net::TcpListener::bind(&address) {
        Ok(l) => l,
        Err(e) => {
            println!("Failed to bind server to {}: {}", address, e);
            std::process::exit(1);
        }
    };

    println!("==================================================");
    println!("   PALSYNC TELEMETRY HOST SERVER RUNNING");
    println!("==================================================");
    println!(" Address  : {}", address);
    println!(" Passcode : {}", passcode);
    println!("==================================================");

    for stream in listener.incoming().flatten() {
        let world_path_clone = world_path.clone();
        let passcode_clone = passcode.clone();
        std::thread::spawn(move || {
            handle_host_connection(stream, &world_path_clone, &passcode_clone);
        });
    }
}

fn run_client_request(
    host_ip_port: &str,
    passcode: &str,
    cmd: &str,
    is_json: bool,
    uid: Option<&str>,
) {
    let mut stream = match std::net::TcpStream::connect(host_ip_port) {
        Ok(s) => s,
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Could not connect to host {}: {}", host_ip_port, e) })
                );
            } else {
                println!("Could not connect to host {}: {}", host_ip_port, e);
            }
            std::process::exit(1);
        }
    };

    let mut request_path = format!("/api/command?cmd={}&passcode={}", cmd, passcode);
    if is_json {
        request_path.push_str("&is_json=true");
    }
    if let Some(u) = uid {
        request_path.push_str(&format!("&uid={}", u));
    }

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        request_path, host_ip_port
    );

    use std::io::Write;
    if let Err(e) = stream.write_all(request.as_bytes()) {
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": format!("Failed to send request: {}", e) })
            );
        } else {
            println!("Failed to send request: {}", e);
        }
        std::process::exit(1);
    }

    let mut response = Vec::new();
    if let Err(e) = stream.read_to_end(&mut response) {
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": format!("Failed to read response: {}", e) })
            );
        } else {
            println!("Failed to read response: {}", e);
        }
        std::process::exit(1);
    }

    let response_str = String::from_utf8_lossy(&response);
    if let Some(body_pos) = response_str.find("\r\n\r\n") {
        let status_line = response_str.lines().next().unwrap_or("");
        let body = &response_str[body_pos + 4..];
        if status_line.contains("200 OK") {
            print!("{}", body);
        } else if status_line.contains("403 Forbidden") {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": "Access Denied: Invalid passcode." })
                );
            } else {
                println!("Access Denied: Invalid passcode.");
            }
            std::process::exit(1);
        } else {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({ "status": "error", "message": format!("Server returned error: {}", status_line) })
                );
            } else {
                println!("Server returned error: {}", status_line);
            }
            std::process::exit(1);
        }
    } else {
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": "Invalid response format from host." })
            );
        } else {
            println!("Invalid response format from host.");
        }
        std::process::exit(1);
    }
}

enum McpFormat {
    McpServers,
    Servers,
    Opencode,
}

fn inject_json_mcp(path: &Path, format: McpFormat, command: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut mcp_config = if path.exists() {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str::<serde_json::Value>(&content)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !mcp_config.is_object() {
        mcp_config = serde_json::json!({});
    }

    if let Some(obj) = mcp_config.as_object_mut() {
        match format {
            McpFormat::Opencode => {
                let mcp_servers = obj.entry("mcp").or_insert_with(|| serde_json::json!({}));
                if let Some(servers_obj) = mcp_servers.as_object_mut() {
                    servers_obj.insert(
                        "palsync".to_string(),
                        serde_json::json!({
                            "type": "local",
                            "command": [command, "mcp"],
                            "enabled": true
                        }),
                    );
                }
            }
            McpFormat::Servers => {
                let mcp_servers = obj
                    .entry("servers")
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(servers_obj) = mcp_servers.as_object_mut() {
                    servers_obj.insert(
                        "palsync".to_string(),
                        serde_json::json!({
                            "type": "stdio",
                            "command": command,
                            "args": ["mcp"]
                        }),
                    );
                }
            }
            McpFormat::McpServers => {
                let mcp_servers = obj
                    .entry("mcpServers")
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(servers_obj) = mcp_servers.as_object_mut() {
                    servers_obj.insert(
                        "palsync".to_string(),
                        serde_json::json!({
                            "command": command,
                            "args": ["mcp"]
                        }),
                    );
                }
            }
        }
    }

    if let Ok(json_str) = serde_json::to_string_pretty(&mcp_config) {
        let _ = std::fs::write(path, json_str);
    }
}

fn inject_marker_block(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut file_content = if path.exists() {
        std::fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    let begin_marker = "<!-- BEGIN PALSYNC RULES — managed by palsync setup -->";
    let end_marker = "<!-- END PALSYNC RULES -->";

    let block = format!("{}\n{}\n{}", begin_marker, content.trim(), end_marker);

    if file_content.contains(begin_marker) {
        if let Some(start) = file_content.find(begin_marker) {
            if let Some(end) = file_content.find(end_marker) {
                let actual_end = end + end_marker.len();
                file_content.replace_range(start..actual_end, &block);
            }
        }
    } else {
        if !file_content.is_empty() && !file_content.ends_with('\n') {
            file_content.push('\n');
        }
        file_content.push_str(&block);
        file_content.push('\n');
    }

    let _ = std::fs::write(path, file_content);
}

fn print_setup_complete(
    name: &str,
    dest_exe: &Path,
    mcp_config: &Path,
    rules_file: &Path,
    skill_file: Option<&Path>,
) {
    println!("==================================================");
    println!("   PALSYNC {} SETUP COMPLETED", name.to_uppercase());
    println!("==================================================");
    println!(" Permanent Exe: {}", dest_exe.display());
    println!(" MCP Config   : {}", mcp_config.display());
    println!(" Rules File   : {}", rules_file.display());
    if let Some(sf) = skill_file {
        println!(" Skill File   : {}", sf.display());
    }
    println!("==================================================");
}

fn run_setup(agent_slug: &str) {
    let home_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "C:\\".to_string());
    let home_path = Path::new(&home_dir);

    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("palagent-ai.exe"));

    let palsync_dir = home_path.join(".palagent-ai");
    if let Err(e) = std::fs::create_dir_all(&palsync_dir) {
        println!("Error creating PalAgent directory: {}", e);
        std::process::exit(1);
    }

    let dest_exe = palsync_dir.join("palagent-ai.exe");
    if current_exe != dest_exe {
        if let Err(e) = std::fs::copy(&current_exe, &dest_exe) {
            println!(
                "Warning: Could not copy executable to permanent folder: {}",
                e
            );
        } else {
            println!(
                "Copied executable to permanent location: {}",
                dest_exe.display()
            );
        }
    }

    let standard_dll_paths = [
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "C:\\Program Files\\Steam\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "D:\\SteamLibrary\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "E:\\SteamLibrary\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
        "F:\\SteamLibrary\\steamapps\\common\\Palworld\\Binaries\\Win64\\oo2core_9_win64.dll",
    ];

    let mut found_dll_path = None;
    if let Some(parent) = current_exe.parent() {
        let paths_to_check = [
            parent.join("oo2core_9_win64.dll"),
            parent.join("..").join("oo2core_9_win64.dll"),
            parent.join("..").join("..").join("oo2core_9_win64.dll"),
        ];
        for path in &paths_to_check {
            if path.exists() {
                found_dll_path = Some(path.to_path_buf());
                break;
            }
        }
    }
    if found_dll_path.is_none() {
        for path_str in &standard_dll_paths {
            let path = Path::new(path_str);
            if path.exists() {
                found_dll_path = Some(path.to_path_buf());
                break;
            }
        }
    }

    if let Some(dll_path) = found_dll_path {
        let dest_dll = palsync_dir.join("oo2core_9_win64.dll");
        if let Err(e) = std::fs::copy(&dll_path, &dest_dll) {
            println!(
                "Warning: Could not copy oo2core_9_win64.dll to default folder: {}",
                e
            );
        } else {
            println!(
                "Copied oo2core_9_win64.dll to permanent location: {}",
                dest_dll.display()
            );
        }
    } else {
        println!("Warning: oo2core_9_win64.dll not found in standard paths. You might need to place it manually.");
    }

    let command_str = dest_exe.to_string_lossy().replace("\\", "/");

    let rule_content = "\n# PalSync Rules\nYou have access to PalSync telemetry and monitor tools via MCP.\nWhen the user asks about Palworld save files, in-game stats, Pals, inventory, bases, or breeding, use the palsync MCP tools to retrieve real-time data instead of guessing.\n";
    let skill_body = "---\nname: palsync\ndescription: Extract telemetry, stats, IVs, breeding combinations, and base camps from Palworld save files.\n---\n\n# PalSync Skill\n\nThis skill allows the agent to interact with the PalSync MCP server and query real-time Palworld statistics.\nUse the `palsync` tools when:\n- The user asks for the status of base camps or Palbox.\n- The user wants to analyze Pal IVs, stats, or passive skills.\n- The user requests breeding combinations.\n- The user needs to locate items in base chests.\n";

    match agent_slug {
        "antigravity-cli" => {
            let gemini_config_dir = home_path.join(".gemini").join("config");
            std::fs::create_dir_all(&gemini_config_dir).ok();

            let mcp_config_path = gemini_config_dir.join("mcp_config.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let agents_md_path = gemini_config_dir.join("AGENTS.md");
            inject_marker_block(&agents_md_path, rule_content);

            let gemini_md_path = home_path.join(".gemini").join("GEMINI.md");
            inject_marker_block(&gemini_md_path, rule_content);

            let skill_dir = gemini_config_dir.join("skills").join("palsync");
            std::fs::create_dir_all(&skill_dir).ok();
            std::fs::write(skill_dir.join("SKILL.md"), skill_body).ok();

            print_setup_complete(
                "Antigravity CLI",
                &dest_exe,
                &mcp_config_path,
                &agents_md_path,
                Some(&skill_dir.join("SKILL.md")),
            );
        }
        "vscode-copilot" => {
            let app_data = std::env::var("APPDATA")
                .unwrap_or_else(|_| format!("{}\\AppData\\Roaming", home_dir));
            let vscode_user_dir = Path::new(&app_data).join("Code").join("User");
            std::fs::create_dir_all(&vscode_user_dir).ok();

            let mcp_config_path = vscode_user_dir.join("mcp.json");
            inject_json_mcp(&mcp_config_path, McpFormat::Servers, &command_str);

            let prompts_dir = vscode_user_dir.join("prompts");
            std::fs::create_dir_all(&prompts_dir).ok();
            let instr_file = prompts_dir.join("palsync.instructions.md");
            let copilot_body = format!("---\napplyTo: \"**\"\n---\n\n{}", rule_content);
            std::fs::write(&instr_file, copilot_body).ok();

            print_setup_complete(
                "VS Code Copilot",
                &dest_exe,
                &mcp_config_path,
                &instr_file,
                None,
            );
        }
        "cursor" => {
            let cursor_dir = home_path.join(".cursor");
            std::fs::create_dir_all(&cursor_dir).ok();

            let mcp_config_path = cursor_dir.join("mcp.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let rule_file = cursor_dir.join("palsync-rules.md");
            let cursor_rules_body = format!("---\nalwaysApply: true\n---\n\n{}", rule_content);
            std::fs::write(&rule_file, cursor_rules_body).ok();

            print_setup_complete("Cursor", &dest_exe, &mcp_config_path, &rule_file, None);
        }
        "windsurf" => {
            let codeium_dir = home_path.join(".codeium").join("windsurf");
            std::fs::create_dir_all(&codeium_dir).ok();

            let mcp_config_path = codeium_dir.join("mcp_config.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let memories_dir = codeium_dir.join("memories");
            std::fs::create_dir_all(&memories_dir).ok();
            let rules_file = memories_dir.join("global_rules.md");
            inject_marker_block(&rules_file, rule_content);

            print_setup_complete("Windsurf", &dest_exe, &mcp_config_path, &rules_file, None);
        }
        "opencode" => {
            let config_dir = home_path.join(".config").join("opencode");
            std::fs::create_dir_all(&config_dir).ok();

            let mcp_config_path = config_dir.join("opencode.json");
            inject_json_mcp(&mcp_config_path, McpFormat::Opencode, &command_str);

            let rules_file = config_dir.join("AGENTS.md");
            inject_marker_block(&rules_file, rule_content);

            print_setup_complete("OpenCode", &dest_exe, &mcp_config_path, &rules_file, None);
        }
        "claude-code" => {
            let claude_dir = home_path.join(".claude");
            std::fs::create_dir_all(&claude_dir).ok();

            let mcp_config_path = claude_dir.join("settings.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            print_setup_complete(
                "Claude Code",
                &dest_exe,
                &mcp_config_path,
                &mcp_config_path,
                None,
            );
        }
        "gemini-cli" => {
            let gemini_dir = home_path.join(".gemini");
            std::fs::create_dir_all(&gemini_dir).ok();

            let mcp_config_path = gemini_dir.join("settings.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let system_md = gemini_dir.join("system.md");
            inject_marker_block(&system_md, rule_content);

            print_setup_complete("Gemini CLI", &dest_exe, &mcp_config_path, &system_md, None);
        }
        "codex" => {
            let codex_dir = home_path.join(".codex");
            std::fs::create_dir_all(&codex_dir).ok();

            let config_toml_path = codex_dir.join("config.toml");

            let mut content = if config_toml_path.exists() {
                std::fs::read_to_string(&config_toml_path).unwrap_or_default()
            } else {
                String::new()
            };

            if !content.contains("[mcp_servers.palsync]") {
                let toml_append = format!(
                    "\n[mcp_servers.palsync]\ncommand = \"{}\"\nargs = [\"mcp\"]\n",
                    command_str
                );
                content.push_str(&toml_append);
                std::fs::write(&config_toml_path, content).ok();
            }

            let instr_file = codex_dir.join("palsync-instructions.md");
            std::fs::write(&instr_file, rule_content).ok();

            print_setup_complete("Codex", &dest_exe, &config_toml_path, &instr_file, None);
        }
        "qwen" => {
            let qwen_dir = home_path.join(".qwen");
            std::fs::create_dir_all(&qwen_dir).ok();

            let mcp_config_path = qwen_dir.join("settings.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let qwen_md = qwen_dir.join("QWEN.md");
            inject_marker_block(&qwen_md, rule_content);

            print_setup_complete("Qwen Code", &dest_exe, &mcp_config_path, &qwen_md, None);
        }
        "kiro" => {
            let kiro_dir = home_path.join(".kiro");
            std::fs::create_dir_all(&kiro_dir).ok();

            let mcp_config_path = kiro_dir.join("settings").join("mcp.json");
            std::fs::create_dir_all(kiro_dir.join("settings")).ok();
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            let steering_file = kiro_dir.join("steering").join("palsync.md");
            std::fs::create_dir_all(kiro_dir.join("steering")).ok();
            std::fs::write(&steering_file, rule_content).ok();

            print_setup_complete(
                "Kiro IDE",
                &dest_exe,
                &mcp_config_path,
                &steering_file,
                None,
            );
        }
        "pi" => {
            let pi_dir = home_path.join(".pi").join("config");
            std::fs::create_dir_all(&pi_dir).ok();

            let mcp_config_path = pi_dir.join("mcp.json");
            inject_json_mcp(&mcp_config_path, McpFormat::McpServers, &command_str);

            print_setup_complete("Pi", &dest_exe, &mcp_config_path, &mcp_config_path, None);
        }
        "kilocode" => {
            let config_dir = home_path.join(".config").join("kilo");
            std::fs::create_dir_all(&config_dir).ok();

            let mcp_config_path = config_dir.join("opencode.json");
            inject_json_mcp(&mcp_config_path, McpFormat::Opencode, &command_str);

            let rules_file = config_dir.join("AGENTS.md");
            inject_marker_block(&rules_file, rule_content);

            print_setup_complete("Kilo Code", &dest_exe, &mcp_config_path, &rules_file, None);
        }
        _ => {
            println!("Error: Unsupported agent slug '{}'.", agent_slug);
            println!("Supported slugs: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode");
            std::process::exit(1);
        }
    }
}

fn run_mcp_loop(world_path: PathBuf) {
    use std::io::{self, BufRead};
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
                                "version": "0.1.0"
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

                    let text_result = match tool_name {
                        "query_time" => execute_command_captured(&world_path, "time", false, None),
                        "query_settings" => {
                            execute_command_captured(&world_path, "settings", false, None)
                        }
                        "search_chest" => execute_command_captured(
                            &world_path,
                            &format!("search-chest:{}", search_query),
                            false,
                            None,
                        ),
                        "query_breeding" => {
                            execute_command_captured(&world_path, "breeding", false, player_uid)
                        }
                        "query_progress" => {
                            execute_command_captured(&world_path, "progress", false, player_uid)
                        }
                        "monitor_pals" => {
                            execute_command_captured(&world_path, "monitor", false, player_uid)
                        }
                        "query_analyzer" => {
                            execute_command_captured(&world_path, "analyzer", false, player_uid)
                        }
                        "query_full" => {
                            execute_command_captured(&world_path, "full", false, player_uid)
                        }
                        _ => format!("Unknown tool: {}", tool_name),
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

fn main() {
    i18n::init(i18n::detect_system_language());

    let args_list: Vec<String> = std::env::args().skip(1).collect();
    let is_json = args_list.iter().any(|arg| arg == "--json");

    let setup_pos = args_list.iter().position(|arg| arg == "setup");
    let mut setup_agent = None;
    if let Some(pos) = setup_pos {
        if pos + 1 < args_list.len() {
            setup_agent = Some(args_list[pos + 1].clone());
        }
    }

    if setup_agent.is_none() && args_list.iter().any(|arg| arg == "--setup-antigravity") {
        setup_agent = Some("antigravity-cli".to_string());
    }

    let has_mcp = args_list.iter().any(|arg| arg == "mcp" || arg == "--mcp");

    if let Some(ref agent) = setup_agent {
        run_setup(agent);
        std::process::exit(0);
    }

    let mut world_path_arg = None;
    let mut skip_next = false;
    for arg in &args_list {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--search-chest"
            || arg == "-c"
            || arg == "--connect"
            || arg == "--passcode"
            || arg == "--player-uid"
            || arg == "--uid"
            || arg == "--port"
        {
            skip_next = true;
            continue;
        }
        if !arg.starts_with("-") {
            world_path_arg = Some(arg.clone());
            break;
        }
    }

    let has_time = args_list.iter().any(|arg| arg == "--time" || arg == "-t");
    let has_settings = args_list
        .iter()
        .any(|arg| arg == "--settings" || arg == "-s");
    let has_breeding = args_list
        .iter()
        .any(|arg| arg == "--breeding" || arg == "-b");
    let has_progress = args_list
        .iter()
        .any(|arg| arg == "--progress" || arg == "-p");
    let has_clean_seeds = args_list.iter().any(|arg| arg == "--clean-seeds");
    let has_monitor = args_list
        .iter()
        .any(|arg| arg == "--monitor" || arg == "-m");
    let has_analyzer = args_list
        .iter()
        .any(|arg| arg == "--analyzer" || arg == "-a");
    let has_list_worlds = args_list
        .iter()
        .any(|arg| arg == "--list-worlds" || arg == "-l");
    let has_select_world = args_list.iter().any(|arg| arg == "--select-world");
    let has_host = args_list.iter().any(|arg| arg == "--host");

    let mut connect_arg = None;
    if let Some(pos) = args_list.iter().position(|arg| arg == "--connect") {
        if pos + 1 < args_list.len() {
            connect_arg = Some(args_list[pos + 1].clone());
        }
    }

    let mut passcode_arg = None;
    if let Some(pos) = args_list.iter().position(|arg| arg == "--passcode") {
        if pos + 1 < args_list.len() {
            passcode_arg = Some(args_list[pos + 1].clone());
        }
    }

    let mut player_uid_arg = None;
    if let Some(pos) = args_list
        .iter()
        .position(|arg| arg == "--player-uid" || arg == "--uid")
    {
        if pos + 1 < args_list.len() {
            player_uid_arg = Some(args_list[pos + 1].clone());
        }
    }

    let mut port_val = 8212;
    if let Some(pos) = args_list.iter().position(|arg| arg == "--port") {
        if pos + 1 < args_list.len() {
            if let Ok(p) = args_list[pos + 1].parse::<u16>() {
                port_val = p;
            }
        }
    }

    let mut search_chest_query = None;
    if let Some(pos) = args_list
        .iter()
        .position(|arg| arg == "--search-chest" || arg == "-c")
    {
        if pos + 1 < args_list.len() {
            search_chest_query = Some(args_list[pos + 1].clone());
        }
    }

    if has_list_worlds {
        let worlds = get_all_detected_worlds();
        if worlds.is_empty() {
            println!("{}", i18n::t("no_worlds_detected"));
            std::process::exit(0);
        }
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
        println!(
            "\n================================================================================"
        );
        std::process::exit(0);
    }

    // Client Mode connection
    if let Some(ref connect_host) = connect_arg {
        let passcode = passcode_arg.unwrap_or_default();
        let cmd = if has_time {
            "time"
        } else if has_settings {
            "settings"
        } else if let Some(ref q) = search_chest_query {
            &format!("search-chest:{}", q)
        } else if has_breeding {
            "breeding"
        } else if has_progress {
            "progress"
        } else if has_clean_seeds {
            "clean-seeds"
        } else if has_monitor {
            "monitor"
        } else if has_analyzer {
            "analyzer"
        } else {
            "full"
        };

        run_client_request(
            connect_host,
            &passcode,
            cmd,
            is_json,
            player_uid_arg.as_deref(),
        );
        std::process::exit(0);
    }

    // MCP Mode execution
    if has_mcp {
        let world_path = match world_path_arg {
            Some(ref p) => PathBuf::from(p),
            None => {
                let worlds = get_all_detected_worlds();
                if worlds.is_empty() {
                    let err_json = serde_json::json!({
                        "status": "error",
                        "message": i18n::t("error_detect_save")
                    });
                    println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                    std::process::exit(1);
                }
                if has_select_world {
                    select_world_interactively(&worlds)
                } else {
                    worlds[0].0.clone()
                }
            }
        };
        run_mcp_loop(world_path);
        std::process::exit(0);
    }

    // Host Mode execution
    if has_host {
        let passcode = passcode_arg.unwrap_or_else(|| {
            let u = uuid::Uuid::new_v4().to_string();
            u[..6].to_ascii_uppercase()
        });

        let world_path = match world_path_arg {
            Some(ref p) => PathBuf::from(p),
            None => {
                let worlds = get_all_detected_worlds();
                if worlds.is_empty() {
                    let err_json = serde_json::json!({
                        "status": "error",
                        "message": i18n::t("error_detect_save")
                    });
                    println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                    std::process::exit(1);
                }
                if has_select_world {
                    select_world_interactively(&worlds)
                } else {
                    worlds[0].0.clone()
                }
            }
        };

        start_host_server(world_path, port_val, passcode);
        std::process::exit(0);
    }

    // Singleplayer Mode execution
    let world_path = match world_path_arg {
        Some(ref p) => PathBuf::from(p),
        None => {
            let worlds = get_all_detected_worlds();
            if worlds.is_empty() {
                let err_json = serde_json::json!({
                    "status": "error",
                    "message": i18n::t("error_detect_save")
                });
                println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                std::process::exit(1);
            }
            if has_select_world {
                select_world_interactively(&worlds)
            } else {
                worlds[0].0.clone()
            }
        }
    };

    if has_time {
        run_time_command(&world_path, is_json);
        std::process::exit(0);
    }
    if has_settings {
        run_settings_command(&world_path, is_json);
        std::process::exit(0);
    }
    if let Some(ref query) = search_chest_query {
        run_search_chest_command(&world_path, query, is_json);
        std::process::exit(0);
    }
    if has_breeding {
        run_breeding_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }
    if has_progress {
        run_progress_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }
    if has_clean_seeds {
        run_clean_seeds_command(&world_path, is_json);
        std::process::exit(0);
    }
    if has_monitor {
        run_monitor_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }
    if has_analyzer {
        run_analyzer_command(&world_path, is_json, player_uid_arg.as_deref());
        std::process::exit(0);
    }

    run_full_command(&world_path, is_json, player_uid_arg.as_deref());
}

fn print_beautiful_report(output: &OutputJson) {
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
