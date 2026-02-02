#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::Command;
use tauri::path::BaseDirectory;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn run_exiftool(app: tauri::AppHandle, args: Vec<String>) -> Result<String, String> {
    let resource_path = app
        .path()
        .resolve("resources/exiftool.exe", BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve resource: {}", e))?;

    let exiftool_dir = resource_path
        .parent()
        .ok_or("Failed to get parent directory")?;

    let mut cmd = Command::new(&resource_path);
    cmd.args(&args);
    cmd.current_dir(exiftool_dir);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute exiftool: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_exiftool])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
