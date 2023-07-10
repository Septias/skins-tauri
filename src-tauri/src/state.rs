use anyhow::anyhow;
use directories::ProjectDirs;
use futures::future::join_all;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use itertools::Itertools;
use reqwest::{Client, Request};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::{
    requests::{
        csgobackpack::{ItemListResponse, MarketItem},
        steam::{
            Asset, FullAsset, ItemPrice, MarketPrice, PriceHistoryResponse, UserInventoryResponse,
        },
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
                    manager: CACacheManager {
                        path: ProjectDirs::from("com", "csgo-stocks", "stocks")
                            .unwrap()
                            .cache_dir()
                            .to_path_buf()
                    },
                    options: None,
                }))
                .build(),
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

    // Get all containers a user has.
    // This deduplicates the items fetched.
    pub async fn fetch_user_containers(
        &self,
        game: usize,
        acc: usize,
    ) -> Result<Vec<FullAsset>, StateError> {
        let assets: Vec<FullAsset> = self.fetch_user_items(game, acc, true).await?;
        let containers: Vec<FullAsset> = assets
            .into_iter()
            .filter(|asset| asset.item_type.as_ref().unwrap().as_str() == "Base Grade Container")
            .collect();
        Ok(containers)
    }

    // Get an asset prices from steam market.
    // Options can contain 'currency', 'appid' and probably more...
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
                .header("Cookie", "steamLoginSecure=76561198083067227%7C%7CeyAidHlwIjogIkpXVCIsICJhbGciOiAiRWREU0EiIH0.eyAiaXNzIjogInI6MEQyMV8yMjcxRkM2Rl8yODU2NyIsICJzdWIiOiAiNzY1NjExOTgwODMwNjcyMjciLCAiYXVkIjogWyAid2ViIiBdLCAiZXhwIjogMTY4OTAxMDQ2OSwgIm5iZiI6IDE2ODAyODI0NjcsICJpYXQiOiAxNjg4OTIyNDY3LCAianRpIjogIjBEMUFfMjJEMjk1Q0ZfNTJEOTEiLCAib2F0IjogMTY4MjUwMTk3NCwgInJ0X2V4cCI6IDE3MDA1NTMxMzQsICJwZXIiOiAwLCAiaXBfc3ViamVjdCI6ICI5NC4yMTcuNDIuMTUzIiwgImlwX2NvbmZpcm1lciI6ICI5NC4yMTcuNDIuMTUzIiB9.W4EamXxKIn2dnatGnBeYDkMk54SQqXwFjdAK1NjwxRQcjf_a2SRyy9FuIjnX-bali3ge_IQM3wScJMIxvvyWAQ; sessionid=68bd30a98948c17378e905c5")
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

    // Get an asset prices from steam market.
    // Options can contain 'currency', 'appid' and probably more...
    pub async fn get_asset_price_histories(
        &self,
        assets: Vec<(usize, String)>,
        options: HashMap<String, String>,
    ) -> Result<HashMap<usize, Result<ItemPrice, StateError>>, StateError> {
        let mut requests = vec![];
        let mut used_options: HashMap<String, String> = HashMap::from_iter([
            ("appid".to_string(), "730".to_string()),
            ("currency".to_string(), "3".to_string()),
            ("country".to_string(), "DE".to_string()),
        ]);
        used_options.extend(options);

        for (asset, market_hash_name) in assets.into_iter() {
            let req = self
                .client
                .get("https://steamcommunity.com/market/pricehistory")
                .query(&used_options)
                .query(&[("market_hash_name".to_string(), market_hash_name)])
                .header("Cookie", "sessionid=34ab75c60a3b9fab97c7ad6b; steamCountry=DE%7Cdc66e9879d6dfd8badf861706ae4120c; steamLoginSecure=76561198083067227%7C%7CeyAidHlwIjogIkpXVCIsICJhbGciOiAiRWREU0EiIH0.eyAiaXNzIjogInI6MEQyQl8yMjcxRkNFNl80OEQ4NiIsICJzdWIiOiAiNzY1NjExOTgwODMwNjcyMjciLCAiYXVkIjogWyAid2ViIiBdLCAiZXhwIjogMTY4ODY4MDA4OSwgIm5iZiI6IDE2Nzk5NTM2NzcsICJpYXQiOiAxNjg4NTkzNjc3LCAianRpIjogIjBEMUFfMjJDOTE4NjVfRjU2NDQiLCAib2F0IjogMTY4MjY3NTA4NSwgInJ0X2V4cCI6IDE3MDA3ODE5NDksICJwZXIiOiAwLCAiaXBfc3ViamVjdCI6ICIxNTYuNjcuMTM2LjcxIiwgImlwX2NvbmZpcm1lciI6ICIxNTYuNjcuMTM2LjcxIiB9.YMH4eWikCZRTddIxvvHX3SowVhJs4VJYfL5u0hDojhQDtk8rxGazYlJy6UJrTjn73dMfY7gwMPGUeA9jFdZXCg")
                .build()?;
            let client = self.client.clone();
            let request = tokio::spawn(async move {
                let price: Result<PriceHistoryResponse, _> = Self::send_request(&client, req).await;
                Ok::<_, StateError>((asset, price))
            });
            requests.push(request);
        }
        let prices = join_all(requests).await;
        let prices = prices
            .into_iter()
            .map(|a| a.unwrap().unwrap())
            .map(|(item, request)| (item, request.map(|ph| ph.prices[ph.prices.len() - 500..].to_vec())))
            .collect();
        Ok(prices)
    }

    pub async fn get_all_csgo_items(&self) -> Result<HashMap<String, MarketItem>, StateError> {
        let resp = self.client.get("https://csgobackpack.net/api/GetItemsList/v2/?no_details=true&no_prices=true").send().await?;
        println!("wat");
        Ok(serde_json::from_str::<ItemListResponse>(&resp.text().await?).unwrap().items_list)
    }

    pub async fn get_all_csgo_containers(&self) -> Result<HashMap<usize, MarketItem>, StateError> {
        let items = self.get_all_csgo_items().await?;
        let chests = items.into_iter().filter_map(|(_name, item)| {
            if let Some(item_name) = &item.item_type {
              if item_name == "Container" {
                  Some((item.classid.clone(), item))
              } else {
                  None
              }
            } else {
              None
            }
        }).collect();
        Ok(chests)
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


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple() {
      let state = State::new();
      let requests = state.get_asset_price_histories(vec![(384801285, "Winter Offensive Weapon Case".to_string())], Default::default()).await.unwrap();
      requests.into_values().for_each(|res| {res.unwrap();});
    }
}

