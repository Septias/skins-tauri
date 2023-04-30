#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod requests;
mod state;
mod string_serializer;

use requests::{steam::{FullAsset, MarketPrice, ItemPrice}, csgobackpack::MarketItem};
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

#[tauri::command]
async fn get_asset_price_history(
    assets: Vec<(usize, String)>,
    options: Option<HashMap<String, String>>,
    state: tauri::State<'_, State>,
) -> Result<HashMap<usize, Result<ItemPrice, StateError>>, StateError> {
    state.get_asset_price_histories(assets, options.unwrap_or_default()).await
}

#[tauri::command]
async fn get_all_csgo_items(state: tauri::State<'_, State>) -> Result<HashMap<String, MarketItem>, StateError> {
    state.get_all_csgo_items().await
}

#[tauri::command]
async fn get_all_csgo_containers(state: tauri::State<'_, State>) -> Result<HashMap<usize, MarketItem>, StateError> {
    state.get_all_csgo_containers().await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State::new();

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_user_containers,
            get_user_items,
            get_asset_prices,
            get_asset_price_history,
            get_all_csgo_items,
            get_all_csgo_containers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
