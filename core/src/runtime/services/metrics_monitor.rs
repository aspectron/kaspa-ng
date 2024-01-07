use crate::imports::*;
use crate::runtime::Service;
pub use futures::{future::FutureExt, select, Future};
use kaspa_metrics_core::{Metric, Metrics, MetricsSnapshot};
#[allow(unused_imports)]
use kaspa_wallet_core::rpc::{NotificationMode, Rpc, RpcCtl, WrpcEncoding};

#[allow(clippy::identity_op)]
pub const MAX_METRICS_SAMPLES: usize = 60 * 60 * 24 * 1; // 1 day

pub struct MetricsService {
    pub application_events: ApplicationEventsChannel,
    pub task_ctl: Channel<()>,
    pub metrics: Arc<Metrics>,
    pub metrics_data: Mutex<HashMap<Metric, Vec<PlotPoint>>>,
    pub samples_since_connection: Arc<AtomicUsize>,
}

impl MetricsService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        let metrics = Arc::new(Metrics::default());
        let metrics_data = Metric::list()
            .into_iter()
            .map(|metric| (metric, Vec::new()))
            .collect::<HashMap<Metric, Vec<_>>>();

        Self {
            application_events,
            task_ctl: Channel::oneshot(),
            metrics,
            metrics_data: Mutex::new(metrics_data),
            samples_since_connection: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn metrics_data(&self) -> MutexGuard<'_, HashMap<Metric, Vec<PlotPoint>>> {
        self.metrics_data.lock().unwrap()
    }

    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    pub fn reset_metrics_data(&self) -> Result<()> {
        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::list().into_iter() {
            metrics_data.insert(metric, Vec::with_capacity(MAX_METRICS_SAMPLES));
        }
        Ok(())
    }

    pub fn ingest_metrics_snapshot(&self, snapshot: Box<MetricsSnapshot>) -> Result<()> {
        let timestamp = snapshot.unixtime_millis;
        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::list().into_iter() {
            let dest = metrics_data.get_mut(&metric).unwrap();
            if dest.is_empty() {
                let y = snapshot.get(&metric);
                let mut timestamp = timestamp - MAX_METRICS_SAMPLES as f64 * 1000.0;
                for _ in 0..(MAX_METRICS_SAMPLES - 1) {
                    dest.push(PlotPoint { x: timestamp, y });

                    timestamp += 1000.0;
                }
            }
            if dest.len() > MAX_METRICS_SAMPLES {
                dest.drain(0..dest.len() - MAX_METRICS_SAMPLES);
            }
            dest.push(PlotPoint {
                x: timestamp,
                y: snapshot.get(&metric),
            });
        }

        self.application_events
            .sender
            .try_send(crate::events::Events::Metrics { snapshot })
            .unwrap();

        self.samples_since_connection.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    pub fn samples_since_connection(&self) -> usize {
        self.samples_since_connection.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Service for MetricsService {
    fn name(&self) -> &'static str {
        "metrics-service"
    }

    async fn attach_rpc(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        let this = self.clone();
        self.metrics
            .register_sink(Arc::new(Box::new(move |snapshot: MetricsSnapshot| {
                if let Err(err) = this.ingest_metrics_snapshot(Box::new(snapshot)) {
                    println!("Error ingesting metrics snapshot: {}", err);
                }
                None
            })));

        self.reset_metrics_data()?;
        self.metrics.start_task().await?;
        self.metrics.set_rpc(Some(rpc_api.clone()));
        Ok(())
    }
    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.metrics.unregister_sink();
        self.metrics.stop_task().await?;
        self.metrics.set_rpc(None);

        Ok(())
    }

    async fn connect_rpc(self: Arc<Self>) -> Result<()> {
        self.samples_since_connection.store(0, Ordering::SeqCst);
        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        Ok(())
    }

    fn terminate(self: Arc<Self>) {}

    async fn join(self: Arc<Self>) -> Result<()> {
        Ok(())
    }
}
