use anyhow::anyhow;
use futures::future::join_all;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex}, fs,
};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    requests::steam::{
        Asset, MarketItem, MarketPrice, PriceHistoryResponse, UserInventoryResponse,
    },
    string_serializer, ACC,
};

#[derive(Serialize, TS, Debug)]
#[ts(export)]
pub struct ChestInfo {
    pub name: String,
    pub market_hash_name: String,
    pub icon_url: String,
    pub amount: usize,
    pub price: MarketPrice,
}

impl ChestInfo {
    fn new(
        asset: Asset,
        items: &HashMap<usize, MarketItem>,
        prices: &HashMap<usize, MarketPrice>,
    ) -> Self {
        let item = items[&asset.classid].clone();
        Self {
            name: item.name,
            icon_url: item.icon_url,
            market_hash_name: item.market_hash_name,
            amount: asset.amount,
            price: prices.get(&asset.classid).unwrap().clone(),
        }
    }
}

#[derive(Error, Debug, Serialize)]
pub enum StateError {
    #[error("No user")]
    NoUser,
    #[error(transparent)]
    #[serde(with = "string_serializer")]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    #[serde(with = "string_serializer")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    #[serde(with = "string_serializer")]
    Other(#[from] anyhow::Error),
}

#[derive(Default)]
pub struct State {
    currency: String,
    game: usize,
    user_id: Option<usize>,
    client: reqwest::Client,
    market_items: Arc<Mutex<HashMap<usize, MarketItem>>>,
    user_inventory: Arc<Mutex<Vec<Asset>>>,
    asset_prices: Arc<Mutex<HashMap<usize, MarketPrice>>>,
}

impl State {
    pub fn new(_user_id: Option<usize>) -> Self {
        Self {
            currency: "eur".to_string(),
            game: 730,
            user_id: Some(ACC),
            ..Default::default()
        }
    }

    pub async fn fetch_user_items(&self) -> Result<Vec<Asset>, StateError> {
        if let Some(acc) = self.user_id {
            /* let resp = self
                .client
                .get(format!(
                    "https://steamcommunity.com/inventory/{}/{}/2",
                    acc, self.game
                ))
                .query(&[("l", "english"), ("count", "5000")])
                .send()
                .await?;

            let t = &resp.text().await?; */
            let t = fs::read_to_string("items.json").unwrap();
            let resp = serde_json::from_str::<UserInventoryResponse>(&t).unwrap();

            if resp.success != 1 {
                StateError::Other(anyhow!("Problem with request"));
            }

            let mut user_items = self.user_inventory.lock().unwrap();
            *user_items = resp.assets;
            *self.market_items.lock().unwrap() = resp
                .descriptions
                .into_iter()
                .map(|desc| (desc.classid, desc))
                .collect();

            Ok(user_items.clone())
        } else {
            Err(StateError::NoUser)
        }
    }

    pub async fn fetch_user_containers(&self) -> Result<Vec<ChestInfo>, StateError> {
        let assets = self.fetch_user_items().await?;
        let deduped = dedup_assets(&assets);
        let containers: Vec<Asset> = deduped
            .into_iter()
            .filter(|asset| {
                self.market_items
                    .lock()
                    .unwrap()
                    .get(&asset.classid)
                    .unwrap()
                    .item_type
                    .as_ref()
                    .unwrap()
                    .as_str()
                    == "Base Grade Container"
            })
            .collect();

        self.update_prices(&containers).await?;

        let items = containers
            .into_iter()
            .map(|asset| {
                ChestInfo::new(
                    asset,
                    &self.market_items.lock().unwrap(),
                    &self.asset_prices.lock().unwrap(),
                )
            })
            .collect();
        Ok(items)
    }

    async fn update_prices(&self, assets: &Vec<Asset>) -> Result<(), StateError> {
        let mut requests = vec![];
        for asset in assets {
            let market_name_hash = {
                let assets = self.market_items.lock().unwrap();
                assets[&asset.classid].market_hash_name.clone()
            };
            let game = self.game.to_string();

            let request = tokio::spawn(async move {
                let client = reqwest::Client::new();
                let resp = client
                    .get("https://steamcommunity.com/market/priceoverview/")
                    .query(&[
                        ("appid", game.as_str()),
                        ("market_hash_name", &market_name_hash),
                        ("currency", "3"),
                    ])
                    .send()
                    .await?;
                  let t = resp.text().await?;
                println!("{t}");
                let price = serde_json::from_str::<MarketPrice>(&t).unwrap();
                Ok::<_, StateError>(price)
            });
            requests.push(request);
        }
        let prices = join_all(requests).await;
        prices
            .into_iter()
            .map(|a| a.unwrap().unwrap())
            .zip(assets)
            .for_each(|(price, asset)| {
                self.asset_prices
                    .lock()
                    .unwrap()
                    .insert(asset.classid, price);
            });
        Ok(())
    }

    pub async fn _get_price_history(marke_hash_name: &str) -> anyhow::Result<PriceHistoryResponse> {
        let url = format!("https://steamcommunity.com/market/pricehistory/?country=DE&currency=3&appid=730&market_hash_name={}", urlencoding::encode(marke_hash_name));
        let resp = reqwest::get(&url).await?;
        Ok(serde_json::from_str(&resp.text().await?).unwrap())
    }
}

// Adds the amounts of all assets with the same classid
// Takes first asset of its sort as template
fn dedup_assets(assets: &Vec<Asset>) -> Vec<Asset> {
    assets
        .iter()
        .fold(HashMap::new(), |mut acc: HashMap<usize, Asset>, c| {
            match acc.get_mut(&c.classid) {
                Some(asset) => {
                    asset.amount += 1;
                }
                None => {
                    acc.insert(c.classid, c.clone());
                }
            };
            acc
        })
        .into_values()
        .collect()
}
