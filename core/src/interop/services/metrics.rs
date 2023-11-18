// use std::time::Duration;

use crate::imports::*;
use crate::interop::runtime::Service;
pub use futures::{future::FutureExt, select, Future};
use kaspa_metrics::{Metric, Metrics, MetricsSnapshot};
#[allow(unused_imports)]
use kaspa_wallet_core::rpc::{NotificationMode, Rpc, RpcCtl, WrpcEncoding};
// use kaspa_wallet_core::{ConnectOptions, ConnectStrategy};

#[allow(clippy::identity_op)]
pub const MAX_METRICS_SAMPLES: usize = 60 * 60 * 24 * 1; // 1 day

pub struct MetricsService {
    pub application_events: ApplicationEventsChannel,
    // pub service_events: Channel<KaspadServiceEvents>,
    pub task_ctl: Channel<()>,
    // pub network: Mutex<Network>,
    // pub wallet: Arc<runtime::Wallet>,
    pub metrics: Arc<Metrics>,
    pub metrics_data: Mutex<HashMap<Metric, Vec<PlotPoint>>>,
}

impl MetricsService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        // create service event channel
        // let service_events = Channel::unbounded();

        let metrics = Arc::new(Metrics::default());
        let metrics_data = Metric::list()
            .into_iter()
            .map(|metric| (metric, Vec::new()))
            .collect::<HashMap<Metric, Vec<_>>>();

        Self {
            application_events,
            // service_events,
            task_ctl: Channel::oneshot(),
            // network: Mutex::new(settings.node.network),
            // wallet: Arc::new(wallet),
            metrics,
            metrics_data: Mutex::new(metrics_data),
        }
    }

    // pub async fn stop_all_services(&self) -> Result<()> {

    //     Ok(())
    // }

    // pub async fn start_all_services(self: &Arc<Self>, rpc: Rpc, network: Network) -> Result<()> {
    //     let rpc_api = rpc.rpc_api().clone();

    //     Ok(())
    // }

    pub fn metrics_data(&self) -> MutexGuard<'_, HashMap<Metric, Vec<PlotPoint>>> {
        self.metrics_data.lock().unwrap()
    }

    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    pub fn reset_metrics_data(&self) -> Result<()> {
        let now = unixtime_as_millis_f64();
        let mut template = Vec::with_capacity(MAX_METRICS_SAMPLES);
        let mut plot_point = PlotPoint {
            x: now - MAX_METRICS_SAMPLES as f64 * 1000.0,
            y: 0.0,
        };
        while template.len() < MAX_METRICS_SAMPLES {
            template.push(plot_point);
            plot_point.x += 1000.0;
        }

        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::list().into_iter() {
            metrics_data.insert(metric, template.clone());
        }
        Ok(())
    }

    pub fn ingest_metrics_snapshot(&self, snapshot: Box<MetricsSnapshot>) -> Result<()> {
        let timestamp = snapshot.unixtime;
        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::list().into_iter() {
            let dest = metrics_data.get_mut(&metric).unwrap();
            if dest.len() > MAX_METRICS_SAMPLES {
                dest.drain(0..dest.len() - MAX_METRICS_SAMPLES);
            }
            // else if dest.len() < MAX_METRICS_SAMPLES {
            //     let mut last_point = dest.last().cloned().unwrap_or_default();
            //     while dest.len() < MAX_METRICS_SAMPLES {
            //         last_point.x += 1000.0;
            //         dest.push(last_point.clone());
            //     }
            // }
            dest.push(PlotPoint {
                x: timestamp,
                y: snapshot.get(&metric),
            });
        }

        // if update_metrics_flag().load(Ordering::SeqCst) {
        self.application_events
            .sender
            .try_send(crate::events::Events::Metrics { snapshot })
            .unwrap();
        // }

        Ok(())
    }
}

#[async_trait]
impl Service for MetricsService {
    async fn attach_rpc(self: Arc<Self>, rpc_api: Arc<dyn RpcApi>) -> Result<()> {
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
        self.metrics.set_rpc(Some(rpc_api));
        Ok(())
    }
    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.metrics.unregister_sink();
        self.metrics.stop_task().await?;
        self.metrics.set_rpc(None);

        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        // let this = self.clone();
        // // let wallet_events = this.wallet.multiplexer().channel();
        // let _application_events_sender = self.application_events.sender.clone();

        // loop {
        //     // println!("loop...");
        //     select! {

        //         msg = this.as_ref().service_events.receiver.recv().fuse() => {

        //             if let Ok(event) = msg {

        //                 match event {

        //                     KaspadServiceEvents::Exit => {
        //                         break;
        //                     }
        //                 }
        //             } else {
        //                 break;
        //             }
        //         }
        //     }
        // }

        // println!("shutting down node manager...");
        // this.stop_all_services().await?;
        // this.task_ctl.send(()).await.unwrap();

        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        // self.service_events
        //     .sender
        //     .try_send(KaspadServiceEvents::Exit)
        //     .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        // self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
