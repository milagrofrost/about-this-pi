use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Window};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessInfo {
    name: String,
    memory_bytes: u64,
    memory: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemInfo {
    title: String,
    version: String,
    hardware: String,
    built_in_memory: String,
    virtual_memory: String,
    largest_unused_block: String,
    processes: Vec<ProcessInfo>,
}

fn format_bytes(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes else {
        return "Unknown".to_string();
    };

    let gib = 1024_f64 * 1024_f64 * 1024_f64;
    let mib = 1024_f64 * 1024_f64;
    let bytes = bytes as f64;

    if bytes >= gib {
        format!("{:.1} GB", bytes / gib)
    } else {
        format!("{:.1} MB", bytes / mib)
    }
}

fn read_os_version() -> String {
    let Ok(raw) = fs::read_to_string("/etc/os-release") else {
        return "Unknown Linux".to_string();
    };

    let mut fields = HashMap::new();

    for line in raw.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        fields.insert(key, value.trim_matches('"').to_string());
    }

    fields
        .get("PRETTY_NAME")
        .or_else(|| fields.get("NAME"))
        .cloned()
        .unwrap_or_else(|| "Unknown Linux".to_string())
}

fn read_meminfo() -> (String, String, String) {
    let Ok(raw) = fs::read_to_string("/proc/meminfo") else {
        return (
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
        );
    };

    let mut values = HashMap::new();

    for line in raw.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };

        let kb = rest
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<u64>().ok());

        if let Some(kb) = kb {
            values.insert(key.to_string(), kb * 1024);
        }
    }

    (
        format_bytes(values.get("MemTotal").copied()),
        format_bytes(Some(values.get("SwapTotal").copied().unwrap_or(0))),
        format_bytes(values.get("MemAvailable").copied()),
    )
}

fn read_first_existing_file(paths: &[&str]) -> Option<String> {
    paths.iter().find_map(|path| {
        fs::read_to_string(path)
            .ok()
            .map(|value| value.trim_matches(char::from(0)).trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn read_hardware_model() -> String {
    read_first_existing_file(&[
        "/proc/device-tree/model",
        "/sys/firmware/devicetree/base/model",
        "/sys/devices/virtual/dmi/id/product_name",
    ])
    .unwrap_or_else(|| "Unknown Linux Computer".to_string())
}

fn read_process_name(pid: &str) -> String {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|name| name.trim().to_string())
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("pid {pid}"))
}

fn read_process_memory(pid: &str) -> u64 {
    let Ok(raw) = fs::read_to_string(format!("/proc/{pid}/status")) else {
        return 0;
    };

    raw.lines()
        .find_map(|line| {
            let rest = line.strip_prefix("VmRSS:")?;
            rest.split_whitespace()
                .next()
                .and_then(|value| value.parse::<u64>().ok())
                .map(|kb| kb * 1024)
        })
        .unwrap_or(0)
}

fn read_top_processes() -> Vec<ProcessInfo> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };

    let mut totals_by_name: HashMap<String, u64> = HashMap::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let pid = file_name.to_string_lossy();

        if !pid.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }

        if !Path::new(&format!("/proc/{pid}")).is_dir() {
            continue;
        }

        let memory_bytes = read_process_memory(&pid);
        if memory_bytes == 0 {
            continue;
        }

        let name = read_process_name(&pid);
        *totals_by_name.entry(name).or_insert(0) += memory_bytes;
    }

    let mut processes = totals_by_name
        .into_iter()
        .map(|(name, memory_bytes)| ProcessInfo {
            name,
            memory_bytes,
            memory: format_bytes(Some(memory_bytes)),
        })
        .collect::<Vec<_>>();

    processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    processes.truncate(10);
    processes
}

#[tauri::command]
fn get_system_info() -> SystemInfo {
    let (built_in_memory, virtual_memory, largest_unused_block) = read_meminfo();

    SystemInfo {
        title: option_env!("ABOUT_THIS_COMPUTER_SYSTEM_NAME")
            .unwrap_or("PiForma OS")
            .to_string(),
        version: read_os_version(),
        hardware: read_hardware_model(),
        built_in_memory,
        virtual_memory,
        largest_unused_block,
        processes: read_top_processes(),
    }
}

#[tauri::command]
fn close_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn start_window_drag(window: Window) {
    let _ = window.start_dragging();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            close_app,
            start_window_drag
        ])
        .run(tauri::generate_context!())
        .expect("failed to run About This PiForma");
}
