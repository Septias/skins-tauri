use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use std::{collections::HashMap, fs};

#[derive(Serialize, Deserialize)]
pub struct ItemStats {
    #[serde(alias = "type")]
    pub item_type: Option<String>,
    pub name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub classid: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ItemListResponse {
    pub success: bool,
    pub currency: String,
    pub items_list: HashMap<String, ItemStats>,
}

pub async fn _get_all_csgo_items() -> anyhow::Result<ItemListResponse> {
    let resp = reqwest::get("http://csgobackpack.net/api/GetItemsList/v2/").await?;
    Ok(serde_json::from_str(&resp.text().await?).unwrap())
}

pub async fn _get_all_container_names() -> anyhow::Result<String> {
    let items = _get_all_csgo_items().await?;
    let items = items
        .items_list
        .values()
        .filter(|item| item.item_type == Some("Container".into()))
        .map(|item| &item.name);
    Ok(items.into_iter().join("\n"))
}

#[derive(Serialize, Deserialize)]
pub struct Asset {
    pub appid: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub assetid: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub classid: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub instanceid: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount: usize,
}

#[derive(Serialize, Deserialize)]
pub struct UserInventoryResponse {
    pub success: usize,
    pub total_inventory_count: usize,
    pub assets: Vec<Asset>,
}

#[derive(Serialize, Deserialize)]
pub struct ItemPriceResponse {
    pub success: bool,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub average_price: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub median_price: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount_sold: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub standard_deviation: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lowest_price: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub highest_price: f64,
    pub first_sale_date: String,
    pub time: String,
    pub icon: Option<String>,
    pub currency: String,
}

pub async fn get_item_price(marke_hash_name: &str) -> anyhow::Result<ItemPriceResponse> {
    let url = format!(
        "http://csgobackpack.net/api/GetItemPrice/?id={}",
        urlencoding::encode(marke_hash_name)
    );
    let resp = reqwest::get(&url).await?;
    Ok(serde_json::from_str(&resp.text().await?).unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct PriceHistoryResponse {
    pub success: bool,
    pub price_prefix: String,
    pub price_suffix: char,
    pub prices: Vec<(String, f64, String)>,
}

pub async fn _get_price_history(marke_hash_name: &str) -> anyhow::Result<PriceHistoryResponse> {
    let url = format!("https://steamcommunity.com/market/pricehistory/?country=DE&currency=3&appid=730&market_hash_name={}", urlencoding::encode(marke_hash_name));
    let resp = reqwest::get(&url).await?;
    Ok(serde_json::from_str(&resp.text().await?).unwrap())
}
