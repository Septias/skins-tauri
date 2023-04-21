use std::{collections::HashMap, fs};

use thiserror::Error;

use crate::requests::{Asset, ItemListResponse, ItemStats, UserInventoryResponse};

#[derive(Error, Debug)]
enum StateError {
    #[error("No user")]
    NoUser,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

struct State {
    currency: String,
    game: usize,
    user_id: Option<usize>,
    client: reqwest::Client,
}

impl State {
    async fn get_user_items(&self) -> Result<Vec<Asset>, StateError> {
        if let Some(acc) = self.user_id {
            let resp = self
                .client
                .get(format!(
                    "https://steamcommunity.com/inventory/{}/{}/2",
                    acc, self.game
                ))
                .query(&[("l", "english", "count", "5000")])
                .send()
                .await?;

            let inventory = serde_json::from_str::<UserInventoryResponse>(&resp.text().await?)?;
            
            // deduplicate items (cases)
            let items = inventory.assets.into_iter().fold(
                HashMap::new(),
                |mut acc: HashMap<usize, Asset>, c| {
                    match acc.get_mut(&c.classid) {
                        Some(asset) => {
                            asset.amount += 1;
                        }
                        None => {
                            acc.insert(c.classid, c);
                        }
                    };
                    acc
                },
            );
            Ok(items.into_values().collect())
        } else {
            Err(StateError::NoUser)
        }
    }

    async fn get_user_containers(&self) -> anyhow::Result<Vec<Asset>> {
        let items = fs::read_to_string("item_list.json")?;
        let items = serde_json::from_str::<ItemListResponse>(&items).unwrap();

        let user_items = self.get_user_items().await?;
        
        let item_list = items
            .items_list
            .into_values()
            .filter(|item| item.item_type == Some("Container".into()))
            .collect::<Vec<_>>();
        Ok(item_list)
    }
}
