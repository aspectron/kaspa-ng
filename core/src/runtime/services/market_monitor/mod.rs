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

#[derive(Clone, Serialize, Deserialize)]
pub struct CurrencyDescriptor {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

pub type CurrencyDescriptorList = Vec<CurrencyDescriptor>;

pub enum MarketMonitorEvents {
    Enable,
    Disable,
    Exit,
}

pub struct MarketMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<MarketMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub is_enabled: AtomicBool,
    pub currencies: Mutex<Option<Vec<String>>>,
    pub provider: Mutex<MarketDataProvider>,
    pub available_currencies: Mutex<Option<Vec<CurrencyDescriptor>>>,
    pub market_price_list: Mutex<Option<Arc<MarketDataMap>>>,
}

impl MarketMonitorService {
    pub fn new(application_events: ApplicationEventsChannel, settings: &Settings) -> Self {
        let currencies = ["usd", "btc"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            is_enabled: AtomicBool::new(settings.market_monitor),
            provider: Mutex::new(MarketDataProvider::default()),
            currencies: Mutex::new(Some(currencies)),
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

    pub fn enable(&self, enable: bool) {
        if enable {
            self.service_events
                .try_send(MarketMonitorEvents::Enable)
                .unwrap();
        } else {
            self.service_events
                .try_send(MarketMonitorEvents::Disable)
                .unwrap();
        }
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
                self.application_events
                    .sender
                    .try_send(Events::Market(MarketUpdate::Price(Arc::new(
                        market_price_list,
                    ))))
                    .unwrap();
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Service for MarketMonitorService {
    fn name(&self) -> &'static str {
        "market-monitor"
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let _application_events_sender = self.application_events.sender.clone();
        let interval = task::interval(Duration::from_secs(POLLING_INTERVAL_SECONDS));
        pin_mut!(interval);

        loop {
            select! {
                _ = interval.next().fuse() => {
                    this.update_market_price_list().await?;
                },

                msg = this.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            MarketMonitorEvents::Enable => {
                                if !this.is_enabled.load(Ordering::SeqCst) {
                                    this.is_enabled.store(true, Ordering::SeqCst);
                                    this.update_market_price_list().await?;
                                }
                            }
                            MarketMonitorEvents::Disable => {
                                this.is_enabled.store(false, Ordering::SeqCst);
                            }
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
        self.service_events
            .sender
            .try_send(MarketMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
