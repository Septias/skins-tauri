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

static CONTAINTERS: [&str; 30] = ["Winter Offensive Weapon Case",
"Operation Phoenix Weapon Case",
"Huntsman Weapon Case",
"Operation Breakout Weapon Case",
"Operation Vanguard Weapon Case",
"Chroma Case",
"Chroma 2 Case",
"Falchion Case",
"Shadow Case",
"Revolver Case",
"Operation Wildfire Case",
"Chroma 3 Case",
"Gamma Case",
"Gamma 2 Case",
"Glove Case",
"Spectrum Case",
"Operation Hydra Case",
"Spectrum 2 Case",
"Clutch Case",
"Horizon Case",
"Danger Zone Case",
"Prisma Case",
"CS20 Case",
"Shattered Web Case",
"Prisma 2 Case",
"Operation Broken Fang Case",
"Fracture Case",
"Snakebite Case",
"Revolution Case",
"Recoil Case"];

#[tauri::command]
async fn get_all_csgo_basic_cases(state: tauri::State<'_, State>) -> Result<HashMap<usize, MarketItem>, StateError> {
    Ok(state.get_all_csgo_containers().await?.into_iter().filter_map(|(_name, item)| {
      if CONTAINTERS.contains(&item.name.as_str()) {
          Some((item.classid.clone(), item))
      } else {
          None
      }
  }).collect())
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
            get_all_csgo_containers,
            get_all_csgo_basic_cases
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
