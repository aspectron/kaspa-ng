use crate::imports::*;
use crate::market::*;

mod coingecko;
mod coinmarketcap;

pub const POLLING_INTERVAL_SECONDS: u64 = 60;

#[derive(Default, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketDataProvider {
    #[default]
    CoinGecko,
    CoinMarketCap,
}

impl MarketDataProvider {
    #[allow(dead_code)]
    async fn fetch_available_currencies(&self) -> Result<CurrencyDescriptorList> {
        match self {
            Self::CoinGecko => coingecko::fetch_available_currencies().await,
            Self::CoinMarketCap => coinmarketcap::fetch_available_currencies().await,
        }
    }

    async fn fetch_market_price_list(&self, currencies: &[&str]) -> Result<MarketDataMap> {
        match self {
            Self::CoinGecko => coingecko::fetch_market_price_list(currencies).await,
            Self::CoinMarketCap => coinmarketcap::fetch_market_price_list(currencies).await,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct MarketMonitorSettings {
    enabled: bool,
    provider: MarketDataProvider,
    currencies: Vec<String>,
}

// struct MarketDataProvider {
//     pub name: String,
//     pub url: String,
// }

// impl MarketDataProvider {
//     pub fn new(name: String, url: String) -> Self {
//         Self { name, url }
//     }

//     pub async fn get(&self) -> Result<serde_json::Value> {
//         let resp = get_json(&self.url).await?;
//         Ok(resp)
//     }
// }

// struct MarketPriceList {
//     pub prices: HashMap<String, MarketPrice>,
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct CurrencyDescriptor {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

pub type CurrencyDescriptorList = Vec<CurrencyDescriptor>;

// pub struct MarketData {}

pub enum MarketMonitorEvents {
    Exit,
}

pub struct MarketMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub plugin_events: Channel<MarketMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub is_enabled: AtomicBool,
    pub currencies: Mutex<Option<Vec<String>>>,
    pub provider: Mutex<MarketDataProvider>,
    pub available_currencies: Mutex<Option<Vec<CurrencyDescriptor>>>,
    pub market_price_list: Mutex<Option<Arc<MarketDataMap>>>,
}

impl MarketMonitorService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        let currencies = ["usd", "btc"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        Self {
            application_events,
            plugin_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            is_enabled: AtomicBool::new(false),
            provider: Mutex::new(MarketDataProvider::default()),
            // ------
            // currencies: Mutex::new(Some(vec!["usd".to_string()])),
            currencies: Mutex::new(Some(currencies)),
            // ------
            // currencies: Mutex::new(None),
            available_currencies: Mutex::new(None),
            market_price_list: Mutex::new(None),
        }
    }

    pub fn currencies(&self) -> Option<Vec<String>> {
        self.currencies.lock().unwrap().clone()
    }

    pub fn provider(&self) -> MarketDataProvider {
        self.provider.lock().unwrap().clone()
    }

    #[allow(dead_code)]
    async fn update_available_currencies(&self) -> Result<()> {
        let available_currencies = self.provider().fetch_available_currencies().await?;
        self.available_currencies
            .lock()
            .unwrap()
            .replace(available_currencies);
        Ok(())
    }

    async fn update_market_price_list(&self) -> Result<()> {
        if let Some(currencies) = self.currencies() {
            let currencies = currencies.iter().map(String::as_str).collect::<Vec<_>>();
            if let Ok(market_price_list) =
                self.provider().fetch_market_price_list(&currencies).await
            {
                // println!("market price list: {:?}", market_price_list);
                // self.market_price_list
                //     .lock()
                //     .unwrap()
                //     .replace(Arc::new(market_price_list));

                self.application_events
                    .sender
                    .try_send(Events::Market(MarketUpdate::Price(Arc::new(
                        market_price_list,
                    ))))
                    .unwrap();
                // println!("market_data: {:?}", market_data);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Service for MarketMonitorService {
    // fn ident(&self) -> &'static str {
    //     "market-monitor"
    // }

    fn name(&self) -> &'static str {
        "market-monitor"
    }

    // fn load(&self, settings: serde_json::Value) -> Result<()> {
    //     let MarketMonitorSettings {
    //         enabled,
    //         provider,
    //         currencies,
    //     } = serde_json::from_value(settings)?;
    //     self.is_enabled.store(enabled, Ordering::SeqCst);
    //     self.currencies.lock().unwrap().replace(currencies);
    //     *self.provider.lock().unwrap() = provider;

    //     Ok(())
    // }

    // fn store(&self) -> Result<Option<serde_json::Value>> {
    //     let settings = MarketMonitorSettings {
    //         enabled: self.is_enabled.load(Ordering::SeqCst),
    //         provider: self.provider.lock().unwrap().clone(),
    //         currencies: self.currencies.lock().unwrap().clone().unwrap_or_default(),
    //     };

    //     Ok(Some(serde_json::to_value(settings)?))
    // }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let _application_events_sender = self.application_events.sender.clone();
        let interval = interval(Duration::from_secs(POLLING_INTERVAL_SECONDS));
        pin_mut!(interval);

        loop {
            select! {
                _ = interval.next().fuse() => {
                    this.update_market_price_list().await?;
                },

                msg = this.as_ref().plugin_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            MarketMonitorEvents::Exit => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        this.task_ctl.send(()).await.unwrap();

        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.plugin_events
            .sender
            .try_send(MarketMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }

    // fn render(&self, ui: &mut Ui) {
    //     ui.label("Market Monitor");

    //     ui.label("TODO - Add Market Monitor Settings");
    // }
}
