use anyhow::anyhow;
use futures::future::join_all;
use http_cache_reqwest::{HttpCache, Cache, CacheMode, CACacheManager};
use itertools::Itertools;
use reqwest::{Client, Request};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
};
use thiserror::Error;

use crate::{
    requests::steam::{
        Asset, FullAsset, MarketPrice, PriceHistoryResponse, UserInventoryResponse,
    },
    string_serializer,
};

#[derive(Error, Debug, Serialize)]
pub enum StateError {
    #[error(transparent)]
    #[serde(with = "string_serializer")]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
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

pub struct State {
    client: ClientWithMiddleware,
}

impl State {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new(Client::new())
            .with(Cache(HttpCache {
              mode: CacheMode::Default,
              manager: CACacheManager::default(),
              options: None,
            }))
            .build()
        }
    }

    // Send a request and return an Error when the response is not 200.
    async fn send_request<T: for<'a> Deserialize<'a>>(
        client: &ClientWithMiddleware,
        req: Request,
    ) -> Result<T, StateError> {
        let resp = client.execute(req).await?;
        if resp.status() != 200 {
            return Err(StateError::Other(anyhow!(
                "The request returned an error response code: {}",
                resp.status()
            )));
        }
        let body = resp.text().await?;
        let serialized = serde_json::from_str::<T>(&body)?;
        Ok(serialized)
    }

    // Get all items a user has.
    pub async fn fetch_user_items(
        &self,
        game: usize,
        acc: usize,
        dedup: bool,
    ) -> Result<Vec<FullAsset>, StateError> {
        let req = self
            .client
            .get(format!(
                "https://steamcommunity.com/inventory/{}/{}/2",
                acc, game
            ))
            .query(&[("l", "english"), ("count", "5000")])
            .build()?;

        println!("{}", req.url());
        let assets: UserInventoryResponse = Self::send_request(&self.client, req).await?;

        let mut item_descriptions: HashMap<usize, _> = assets
            .descriptions
            .into_iter()
            .map(|desc| (desc.classid, desc))
            .collect();

        let mut assets = assets.assets;
        if dedup {
            assets = dedup_assets(&assets);
        }
        Ok(assets
            .into_iter()
            .map(|asset| {
                let classid = item_descriptions.remove(&asset.classid).unwrap();
                asset.hydrate(classid)
            })
            .collect_vec())
    }

    // Get all containers the user has.
    // This deduplicates the items fetched.
    pub async fn fetch_user_containers(
        &self,
        game: usize,
        acc: usize,
    ) -> Result<Vec<FullAsset>, StateError> {
        let assets: Vec<FullAsset> = self.fetch_user_items(game, acc, true).await?;
        let containers: Vec<FullAsset> = assets
            .into_iter()
            .filter(|asset| {
                  asset.item_type
                    .as_ref()
                    .unwrap()
                    .as_str()
                    == "Base Grade Container"
            })
            .collect();
        Ok(containers)
    }

    // Get a asset prices from steam market
    // Options can contain 'currency' 'appid' and probably more...
    pub async fn get_asset_prices(
        &self,
        assets: Vec<(usize, String)>,
        options: HashMap<String, String>,
    ) -> Result<HashMap<usize, Result<MarketPrice, StateError>>, StateError> {
        let mut requests = vec![];
        let mut used_options: HashMap<String, String> = HashMap::from_iter([
            ("appid".to_string(), "730".to_string()),
            ("currency".to_string(), "3".to_string()),
        ]);
        used_options.extend(options);

        for (asset, market_hash_name) in assets.into_iter() {
            let req = self
                .client
                .get("https://steamcommunity.com/market/priceoverview/")
                .query(&used_options)
                .query(&[("market_hash_name".to_string(), market_hash_name)])
                .build()?;
            let client = self.client.clone();
            let request = tokio::spawn(async move {
                let price: Result<MarketPrice, _> = Self::send_request(&client, req).await;
                Ok::<_, StateError>((asset, price))
            });
            requests.push(request);
        }
        let prices = join_all(requests).await;
        let prices = prices.into_iter().map(|a| a.unwrap().unwrap()).collect();
        Ok(prices)
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
