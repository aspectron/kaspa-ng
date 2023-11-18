use super::*;
// use crate::imports::*;
use workflow_http::get_json;

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

#[derive(Default, Serialize, Deserialize)]
struct CoinGeckoSimplePrice {
    kaspa: Option<HashMap<String, f64>>,
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

impl From<CoinGeckoSimplePrice> for MarketPriceMap {
    fn from(data: CoinGeckoSimplePrice) -> Self {
        let mut prices = HashMap::new();
        if let Some(kaspa) = data.kaspa {
            prices = group_by_currency_prefix(&kaspa);
        }
        prices
    }
}

pub async fn fetch_available_currencies() -> Result<CurrencyDescriptorList> {
    let url = "https://api.coingecko.com/api/v3/coins/list";
    let available_currencies = get_json::<CurrencyDescriptorList>(url).await?;
    Ok(available_currencies)
}

pub async fn fetch_market_price_list(currencies: &[&str]) -> Result<MarketPriceMap> {
    let market_data = CoinGeckoSimplePrice::get(currencies).await?;
    Ok(market_data.into())
}

fn group_by_currency_prefix(data: &HashMap<String, f64>) -> MarketPriceMap {
    let mut grouped_data: MarketPriceMap = HashMap::new();

    for (coin, info) in data.iter() {
        let parts: Vec<&str> = coin.split('_').collect();
        let currency_prefix = parts[0].to_lowercase();
        let suffix = parts.last().map(|suffix| suffix.to_lowercase());
        let existing_data = grouped_data.entry(currency_prefix.clone()).or_default();
        match suffix.as_deref() {
            None => existing_data.price = Some(*info),
            Some("market_cap") => existing_data.market_cap = Some(*info),
            Some("24h_vol") => existing_data.volume = Some(*info),
            Some("24h_change") => existing_data.change = Some(*info),
            _ => (),
        }
    }

    grouped_data
}
