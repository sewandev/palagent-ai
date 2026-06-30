use flate2::read::ZlibDecoder;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

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

pub fn decompress_oodle(data: &[u8], uncompressed_len: usize) -> Result<Vec<u8>, String> {
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

pub fn decompress_gvas(path: &Path) -> Result<Vec<u8>, String> {
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
