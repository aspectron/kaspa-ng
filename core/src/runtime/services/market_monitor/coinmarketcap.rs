use super::*;
use std::collections::hash_map::Entry;
use workflow_http::get_json;

#[derive(Default, Serialize, Deserialize)]
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
        Ok(get_json::<Self>(url).await?)
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
    Ok(market_data.into())
}

fn group_by_currency_prefix(data: &AHashMap<String, f64>) -> MarketDataMap {
    let mut grouped_data: MarketDataMap = AHashMap::new();

    for (symbol, info) in data.iter() {
        let parts: Vec<&str> = symbol.split('_').collect();
        let currency_prefix = parts[0].to_lowercase();
        let suffix = parts.last().map(|suffix| suffix.to_lowercase());
        let data = match grouped_data.entry(currency_prefix.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(MarketData::new(symbol.as_str())),
        };

        match suffix.as_deref() {
            None => data.price = *info,
            Some("market_cap") => data.market_cap = *info,
            Some("24h_vol") => data.volume = *info,
            Some("24h_change") => data.change = *info,
            _ => (),
        }
    }

    grouped_data
}
