#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod requests;
mod state;
mod string_serializer;

use requests::steam::{FullAsset, MarketPrice};
use state::{State, StateError};
use std::collections::HashMap;
use tokio;

#[tauri::command]
async fn get_user_containers(
    game: usize,
    user: String,
    state: tauri::State<'_, State>,
) -> Result<Vec<FullAsset>, StateError> {
    state.fetch_user_containers(game, user.parse().unwrap()).await
}

#[tauri::command]
async fn get_user_items(
    game: usize,
    user: String,
    dedup: bool,
    state: tauri::State<'_, State>,
) -> Result<Vec<FullAsset>, StateError> {
    state.fetch_user_items(game, user.parse().unwrap(), dedup).await
}

#[tauri::command]
async fn get_asset_prices(
    assets: Vec<(usize, String)>,
    options: Option<HashMap<String, String>>,
    state: tauri::State<'_, State>,
) -> Result<HashMap<usize, Result<MarketPrice, StateError>>, StateError> {
    state.get_asset_prices(assets, options.unwrap_or_default()).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State::new();

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_user_containers,
            get_user_items,
            get_asset_prices
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
