use crate::imports::*;

#[derive(Default, Debug)]
pub struct MarketData {
    pub price: Option<f64>,
    pub market_cap: Option<f64>,
    pub volume: Option<f64>,
    pub change: Option<f64>,
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
