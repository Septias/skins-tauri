#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod requests;
mod state;
mod string_serializer;

use state::{ State, StateError};
use tokio;
use requests::steam::Asset;


const ACC: usize = 76561198083067227;

#[tauri::command]
async fn get_user_containers(game: usize, user: usize, state: tauri::State<'_, State>) -> Result<Vec<Asset>, StateError> {
    state.fetch_user_containers(game, user).await
}

#[tauri::command]
async fn get_user_items(game: usize, user: usize, state: tauri::State<'_, State>) -> Result<Vec<Asset>, StateError> {
    state.fetch_user_containers(game, user).await
}

#[tauri::command]
async fn get_item_prices(game: usize, user: usize, state: tauri::State<'_, State>) -> Result<Vec<Asset>, StateError> {
    state.fetch_user_containers(game, user).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State::new(None);
    //state.fetch_user_containers().await?;
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_user_containers])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
