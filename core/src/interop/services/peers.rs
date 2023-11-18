// use std::time::Duration;

use crate::imports::*;
use crate::interop::runtime::Service;
pub use futures::{future::FutureExt, select, Future};
use kaspa_rpc_core::RpcPeerInfo;
// use kaspa_metrics::{Metric, Metrics, MetricsSnapshot};
#[allow(unused_imports)]
use kaspa_wallet_core::rpc::{NotificationMode, Rpc, RpcCtl, WrpcEncoding};
// use kaspa_wallet_core::{ConnectOptions, ConnectStrategy};

// #[allow(clippy::identity_op)]
pub const PEER_POLLING_INTERVAL: usize = 1; // 1 sec

pub enum PeerMonitorEvents {
    Exit,
}

pub struct PeerMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<PeerMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    pub peer_info: Mutex<Option<Arc<Vec<RpcPeerInfo>>>>,
}

impl PeerMonitorService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            rpc_api: Mutex::new(None),
            peer_info: Mutex::new(None),
        }
    }

    pub fn rpc_api(&self) -> Option<Arc<dyn RpcApi>> {
        self.rpc_api.lock().unwrap().clone()
    }

    pub fn peer_info(&self) -> Option<Arc<Vec<RpcPeerInfo>>> {
        self.peer_info.lock().unwrap().clone()
    }
}

#[async_trait]
impl Service for PeerMonitorService {
    async fn attach_rpc(self: Arc<Self>, rpc_api: Arc<dyn RpcApi>) -> Result<()> {
        self.rpc_api.lock().unwrap().replace(rpc_api);
        Ok(())
    }

    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.rpc_api.lock().unwrap().take();
        self.peer_info.lock().unwrap().take();

        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        // let wallet_events = this.wallet.multiplexer().channel();
        let _application_events_sender = self.application_events.sender.clone();

        let interval = interval(Duration::from_secs(1));
        pin_mut!(interval);

        loop {
            // println!("loop...");
            select! {
                _ = interval.next().fuse() => {
                    if let Some(rpc_api) = this.rpc_api() {
                        if let Ok(resp) = rpc_api.get_connected_peer_info().await {
                            this.peer_info.lock().unwrap().replace(Arc::new(resp.peer_info));
                        }
                    }
                },

                msg = this.as_ref().service_events.receiver.recv().fuse() => {

                    if let Ok(event) = msg {

                        match event {

                            PeerMonitorEvents::Exit => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        println!("shutting down peer monitor...");
        // this.stop_all_services().await?;
        this.task_ctl.send(()).await.unwrap();

        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(PeerMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
