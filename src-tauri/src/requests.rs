pub mod csgobackpack {
    use serde::{Deserialize, Serialize};
    use serde_aux::prelude::deserialize_number_from_string as str_to_num;
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize)]
    pub struct MarketItem {
        #[serde(alias = "type")]
        pub item_type: Option<String>,
        pub name: String,
        #[serde(deserialize_with = "str_to_num")]
        pub classid: usize,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ItemListResponse {
        pub success: bool,
        pub currency: String,
        pub items_list: HashMap<String, MarketItem>,
    }

    pub async fn _get_all_csgo_items() -> anyhow::Result<ItemListResponse> {
        let resp = reqwest::get("http://csgobackpack.net/api/GetItemsList/v2/").await?;
        Ok(serde_json::from_str(&resp.text().await?).unwrap())
    }

    #[derive(Serialize, Deserialize)]
    pub struct ItemPriceResponse {
        pub success: bool,
        #[serde(deserialize_with = "str_to_num")]
        pub average_price: f32,
        #[serde(deserialize_with = "str_to_num")]
        pub median_price: f32,
        #[serde(deserialize_with = "str_to_num")]
        pub amount_sold: usize,
        #[serde(deserialize_with = "str_to_num")]
        pub standard_deviation: f32,
        #[serde(deserialize_with = "str_to_num")]
        pub lowest_price: f32,
        #[serde(deserialize_with = "str_to_num")]
        pub highest_price: f32,
        pub first_sale_date: String,
        pub time: String,
        pub icon: Option<String>,
        pub currency: String,
    }

    pub async fn _get_item_price(marke_hash_name: &str) -> anyhow::Result<ItemPriceResponse> {
        let url = format!(
            "http://csgobackpack.net/api/GetItemPrice/?id={}",
            urlencoding::encode(marke_hash_name)
        );
        let resp = reqwest::get(&url).await?;
        Ok(serde_json::from_str(&resp.text().await?).unwrap())
    }
}

pub mod steam {
    use serde::{Deserialize, Deserializer, Serialize};
    use serde_aux::prelude::deserialize_number_from_string as str_to_num;
    use ts_rs::TS;
    use std::{
        fmt::{self, Display},
        str::FromStr,
    };
    #[derive(Serialize, Deserialize, Clone)]
    pub struct Asset {
        pub appid: usize,
        #[serde(deserialize_with = "str_to_num")]
        pub assetid: usize,
        #[serde(deserialize_with = "str_to_num")]
        pub classid: usize,
        #[serde(deserialize_with = "str_to_num")]
        pub instanceid: usize,
        #[serde(deserialize_with = "str_to_num")]
        pub amount: usize,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct MarketItem {
        #[serde(alias = "type")]
        pub item_type: Option<String>,
        pub name: String,
        #[serde(deserialize_with = "str_to_num")]
        pub classid: usize,
        pub icon_url: String,
        pub market_hash_name: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct UserInventoryResponse {
        pub success: usize,
        pub total_inventory_count: usize,
        pub assets: Vec<Asset>,
        pub descriptions: Vec<MarketItem>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PriceHistoryResponse {
        pub success: bool,
        pub price_prefix: String,
        pub price_suffix: char,
        pub prices: Vec<(String, f32, String)>,
    }

    #[derive(Deserialize, TS, Serialize, Clone, Debug)]
    #[ts(export)]
    pub struct MarketPrice {
        pub success: bool,
        #[serde(deserialize_with = "deserialize_euro")]
        pub median_price: f32,
        #[serde(deserialize_with = "deserialize_comma_separated")]
        pub volume: usize,
        #[serde(deserialize_with = "deserialize_euro")]
        pub lowest_price: f32,
    }

    pub fn deserialize_euro<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr + serde::Deserialize<'de>,
        <T as FromStr>::Err: Display + fmt::Debug,
    {
        let mut string = String::deserialize(deserializer).unwrap();
        string.pop();
        let string = string.replace(',', ".").replace('-', "0");
        Ok(string.parse::<T>().unwrap())
    }

    pub fn deserialize_comma_separated<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr + serde::Deserialize<'de>,
        <T as FromStr>::Err: Display + fmt::Debug,
    {
        let string = String::deserialize(deserializer).unwrap();
        Ok(string.replace(',', "").parse::<T>().unwrap())
    }
}
