//! Seed export/import commands (D-10, D-12).
//! Stub created by plan 06-05 for compilation; full implementation provided by plan 06-04.

use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::state::AppState;

/// Tauri command: Export a seed as pretty-printed JSON to a user-chosen file path.
#[tauri::command]
pub async fn export_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    seed_id: String,
) -> Result<(), String> {
    Err("Not yet implemented — will be provided by plan 06-04".to_string())
}

/// Tauri command: Import a seed from a JSON file.
#[tauri::command]
pub async fn import_seed(state: State<'_, Mutex<AppState>>, app: AppHandle) -> Result<(), String> {
    Err("Not yet implemented — will be provided by plan 06-04".to_string())
}
