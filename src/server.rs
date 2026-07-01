use crate::commands::{
    run_analyzer_command, run_breeding_command, run_clean_seeds_command, run_full_command,
    run_monitor_command, run_search_chest_command, run_settings_command, run_time_command,
};
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};

pub fn execute_command_captured(
    world_path: &Path,
    cmd: &str,
    is_json: bool,
    target_uid: Option<&str>,
) -> String {
    crate::output::capture_output(|| match cmd {
        "time" => run_time_command(world_path, is_json),
        "settings" => run_settings_command(world_path, is_json),
        "breeding" => run_breeding_command(world_path, is_json, target_uid),
        "progress" => crate::commands::run_progress_command(world_path, is_json, target_uid),
        "clean-seeds" => run_clean_seeds_command(world_path, is_json),
        "monitor" => run_monitor_command(world_path, is_json, target_uid),
        "analyzer" => run_analyzer_command(world_path, is_json, target_uid),
        c if c.starts_with("search-chest:") => {
            let query = &c["search-chest:".len()..];
            run_search_chest_command(world_path, query, is_json);
        }
        "list-worlds" => {
            let worlds = crate::utils::get_all_detected_worlds();
            if worlds.is_empty() {
                println!("{}", crate::i18n::t("no_worlds_detected"));
            } else {
                println!("\n=== {} ===\n", crate::i18n::t("list_worlds_title"));
                println!(" {}", crate::i18n::t("list_worlds_header"));
                println!("{}", "-".repeat(80));
                for (idx, (path, modified)) in worlds.iter().enumerate() {
                    let datetime: chrono::DateTime<chrono::Local> = (*modified).into();
                    let world_name = crate::utils::get_world_name(path);
                    let game_mode_key = crate::utils::detect_game_mode(path);
                    let game_mode = crate::i18n::t(&game_mode_key);
                    println!(
                        " [{}] | {} | {} | {} | {}",
                        idx + 1,
                        datetime.format("%Y-%m-%d %H:%M:%S"),
                        game_mode,
                        world_name,
                        path.display()
                    );
                }
                println!(
                    "\n================================================================================"
                );
            }
        }
        "full" => {
            run_full_command(world_path, is_json, target_uid);
        }
        _ => println!("Unknown command"),
    })
}

pub fn send_http_response(
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

pub fn handle_host_connection(mut stream: std::net::TcpStream, world_path: &Path, passcode: &str) {
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

pub fn start_host_server(world_path: PathBuf, port: u16, passcode: String) {
    let address = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&address) {
        Ok(l) => l,
        Err(e) => {
            println!("Failed to bind server to {}: {}", address, e);
            std::process::exit(1);
        }
    };

    println!("==================================================");
    println!("   PALAGENT-AI TELEMETRY HOST SERVER RUNNING");
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

pub fn run_client_request(
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

pub fn execute_command_remote(
    host_ip_port: &str,
    passcode: &str,
    cmd: &str,
    uid: Option<&str>,
) -> String {
    let mut stream = match std::net::TcpStream::connect(host_ip_port) {
        Ok(s) => s,
        Err(e) => return format!("Error: Could not connect to host {}: {}", host_ip_port, e),
    };

    let mut request_path = format!("/api/command?cmd={}&passcode={}", cmd, passcode);
    request_path.push_str("&is_json=true");
    if let Some(u) = uid {
        request_path.push_str(&format!("&uid={}", u));
    }

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        request_path, host_ip_port
    );

    use std::io::Write;
    if let Err(e) = stream.write_all(request.as_bytes()) {
        return format!("Error: Failed to send request: {}", e);
    }

    let mut response = Vec::new();
    if let Err(e) = stream.read_to_end(&mut response) {
        return format!("Error: Failed to read response: {}", e);
    }

    let response_str = String::from_utf8_lossy(&response);
    if let Some(body_pos) = response_str.find("\r\n\r\n") {
        let status_line = response_str.lines().next().unwrap_or("");
        let body = &response_str[body_pos + 4..];
        if status_line.contains("200 OK") {
            body.to_string()
        } else if status_line.contains("403 Forbidden") {
            "Error: Access Denied: Invalid passcode.".to_string()
        } else {
            format!("Error: Server returned error: {}", status_line)
        }
    } else {
        "Error: Invalid response format from host.".to_string()
    }
}
