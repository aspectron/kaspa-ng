use super::*;
use std::collections::hash_map::Entry;

// https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd%2Ccny&include_market_cap=true&include_24hr_vol=true&include_24hr_change=true
// {
//     "kaspa": {
//       "usd": 0.137395,
//       "usd_market_cap": 2954668910.049152,
//       "usd_24h_vol": 138844602.78193888,
//       "usd_24h_change": 16.77712212157855,
//       "cny": 0.990717,
//       "cny_market_cap": 21305231109.691406,
//       "cny_24h_vol": 1001166777.2797261,
//       "cny_24h_change": 16.744741148538786
//     }
//   }

// https://api.coingecko.com/api/v3/coins/list
// [
//   {
//     "id": "01coin",
//     "symbol": "zoc",
//     "name": "01coin"
//   },

#[derive(Default, Debug, Serialize, Deserialize)]
struct CoinGeckoSimplePrice {
    kaspa: Option<AHashMap<String, f64>>,
}

impl CoinGeckoSimplePrice {
    pub async fn get(currencies: &[&str]) -> Result<Self> {
        let ids = "kaspa";
        let currencies = currencies
            .iter()
            .map(|currency| currency.to_lowercase())
            .collect::<Vec<_>>()
            .join("%2C");
        let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={ids}&vs_currencies={currencies}&include_market_cap=true&include_24hr_vol=true&include_24hr_change=true");
        Ok(http::get_json::<Self>(url).await?)
    }
}

impl From<CoinGeckoSimplePrice> for MarketDataMap {
    fn from(data: CoinGeckoSimplePrice) -> Self {
        let mut prices = AHashMap::new();
        if let Some(kaspa) = data.kaspa {
            prices = group_by_currency_prefix(&kaspa);
        }
        prices
    }
}

pub async fn fetch_available_currencies() -> Result<CurrencyDescriptorList> {
    let url = "https://api.coingecko.com/api/v3/coins/list";
    let available_currencies = http::get_json::<CurrencyDescriptorList>(url).await?;
    Ok(available_currencies)
}

pub async fn fetch_market_price_list(currencies: &[&str]) -> Result<MarketDataMap> {
    let market_data = CoinGeckoSimplePrice::get(currencies).await?;
    // println!("market_data: {:?}", market_data);

    Ok(market_data.into())
}

fn group_by_currency_prefix(data: &AHashMap<String, f64>) -> MarketDataMap {
    let mut grouped_data: MarketDataMap = AHashMap::new();

    for (tag, info) in data.iter() {
        let mut parts: Vec<&str> = tag.split('_').collect();
        if parts.is_empty() {
            continue;
        }
        let symbol = parts.remove(0).to_lowercase();
        let suffix = parts.join("_");
        let data = match grouped_data.entry(symbol.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(MarketData::new(symbol.as_str())),
        };

        match suffix.as_str() {
            "" => data.price = *info,
            "market_cap" => data.market_cap = *info,
            "24h_vol" => data.volume = *info,
            "24h_change" => data.change = *info,
            _ => (),
        }
    }

    grouped_data
}
