use crate::imports::*;

// pub const FEERATE_POLLING_INTERVAL_SECONDS: u64 = 10;
pub const FEERATE_POLLING_INTERVAL_SECONDS: u64 = 3;

pub enum FeerateMonitorEvents {
    Enable,
    Disable,
    Fetch,
    Exit,
}

pub struct FeerateMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<FeerateMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    // pub feerate: Mutex<Option<Arc<RpcFeeEstimate>>>,
    pub is_enabled: Arc<AtomicBool>,
}

impl FeerateMonitorService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            rpc_api: Mutex::new(None),
            // feerate: Mutex::new(None),
            is_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn rpc_api(&self) -> Option<Arc<dyn RpcApi>> {
        self.rpc_api.lock().unwrap().clone()
    }

    // pub fn peer_info(&self) -> Option<Arc<Vec<RpcPeerInfo>>> {
    //     self.peer_info.lock().unwrap().clone()
    // }

    pub fn enable(&self) {
        self.service_events
            .sender
            .try_send(FeerateMonitorEvents::Enable)
            .unwrap();
    }

    pub fn disable(&self) {
        self.service_events
            .sender
            .try_send(FeerateMonitorEvents::Disable)
            .unwrap();
    }

    async fn fetch(self: &Arc<Self>) -> Result<()> {
        if let Some(rpc_api) = self.rpc_api() {
            if let Ok(resp) = rpc_api.get_fee_estimate().await {
                // println!("{}",resp.priority_bucket.feerate);
                // let feerate = Arc::new(resp);
                // self.feerate.lock().unwrap().replace(feerate.clone());
                self.application_events
                    .send(Events::Feerate {
                        feerate: Some(Arc::new(resp)),
                    })
                    .await
                    .unwrap();
            }
        }

        Ok(())
    }

    async fn clear(&self) {
        self.application_events
            .send(Events::Feerate { feerate: None })
            .await
            .unwrap();
    }
}

#[async_trait]
impl Service for FeerateMonitorService {
    fn name(&self) -> &'static str {
        "feerate-monitor"
    }

    async fn attach_rpc(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        self.rpc_api.lock().unwrap().replace(rpc_api.clone());
        Ok(())
    }

    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.rpc_api.lock().unwrap().take();
        self.clear().await;

        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        // let this = self.clone();
        // let application_events_sender = self.application_events.sender.clone();

        let interval = task::interval(Duration::from_secs(FEERATE_POLLING_INTERVAL_SECONDS));
        pin_mut!(interval);

        loop {
            select! {
                _ = interval.next().fuse() => {
                    if !self.is_enabled.load(Ordering::Relaxed) {
                        continue;
                    }

                    self.fetch().await.unwrap();
                },
                msg = self.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            FeerateMonitorEvents::Fetch => {
                                self.fetch().await.unwrap();
                            }
                            FeerateMonitorEvents::Enable => {
                                self.is_enabled.store(true, Ordering::Relaxed);
                                self.fetch().await.unwrap();
                            }
                            FeerateMonitorEvents::Disable => {
                                self.is_enabled.store(false, Ordering::Relaxed);
                                // self.feerate.lock().unwrap().take();
                                self.clear().await;
                            }
                            FeerateMonitorEvents::Exit => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        self.task_ctl.send(()).await.unwrap();
        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(FeerateMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
