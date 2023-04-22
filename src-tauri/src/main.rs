#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use state::{ChestInfo, State, StateError};
use tokio;

mod requests;
mod state;
mod string_serializer;

const ACC: usize = 76561198083067227;

#[tauri::command]
async fn get_user_containers(state: tauri::State<'_, State>) -> Result<Vec<ChestInfo>, StateError> {
    state.fetch_user_containers().await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State::new(None);
    state.fetch_user_containers().await?;
    /* tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_user_containers])
        .run(tauri::generate_context!())
        .expect("error while running tauri application"); */
    Ok(())
}
