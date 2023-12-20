use crate::imports::*;

#[derive(Debug)]
pub struct MarketData {
    pub price: f64,
    pub market_cap: f64,
    pub volume: f64,
    pub change: f64,
    pub precision: usize,
}

impl MarketData {
    pub fn new(symbol: &str) -> Self {
        let precision = precision_from_symbol(symbol);
        Self {
            price: 0.0,
            market_cap: 0.0,
            volume: 0.0,
            change: 0.0,
            precision,
        }
    }
}

pub type MarketDataMap = AHashMap<String, MarketData>;

#[derive(Default, Debug)]
pub struct Ohlc {}

pub type OhlcMap = AHashMap<String, Ohlc>;

#[derive(Default, Debug)]
pub struct Market {
    pub price: Option<Arc<MarketDataMap>>,
    pub ohlc: Option<Arc<OhlcMap>>,
}

#[derive(Clone, Debug)]
pub enum MarketUpdate {
    Price(Arc<MarketDataMap>),
    Ohlc(Arc<OhlcMap>),
}
