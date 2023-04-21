#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use itertools::Itertools;
use requests::{get_item_price, ItemStats};
use std::{fs, sync::Arc};
use tokio;

mod requests;
mod state;

const ACC: &str = "76561198083067227";

#[tauri::command]
async fn get_user_containers() -> anyhow::Result<Vec<(ItemStats, usize)>> {
    todo!()
}

#[tauri::command]
async fn update_root(ilias: tauri::State<'_, Arc<usize>>) -> anyhow::Result<()> {
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let user_containers = get_user_containers().await?;

    let mut user_cointainer_amount_prices = Vec::new();
    for (container, amount) in &user_containers {
        let price = get_item_price(&urlencoding::encode(&container.name).to_string()).await?;
        user_cointainer_amount_prices.push((container, amount, price.average_price))
    }

    let item_list = user_cointainer_amount_prices
        .iter()
        .map(|(itemstat, amount, price)| format!("{price} x {amount} x {}", itemstat.name))
        .join("\n");

    println!(
        "total chest value: {}€",
        user_cointainer_amount_prices
            .iter()
            .fold(0f64, |mut acc, (_item, amount, price)| {
                acc += price * f64::from((**amount) as i32);
                acc
            })
    );
    fs::write("amountxitemxprice.txt", item_list).unwrap();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
