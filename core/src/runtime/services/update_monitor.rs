use crate::imports::*;

pub const UPDATE_POLLING_INTERVAL_SECONDS: u64 = 60 * 60 * 12;

pub enum UpdateMonitorEvents {
    Enable,
    Disable,
    Exit,
}

pub struct UpdateMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<UpdateMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    pub is_enabled: Arc<AtomicBool>,
}

impl UpdateMonitorService {
    pub fn new(application_events: ApplicationEventsChannel, settings: &Settings) -> Self {
        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            rpc_api: Mutex::new(None),
            is_enabled: Arc::new(AtomicBool::new(settings.update_monitor)),
        }
    }

    pub fn enable(&self, enable: bool) {
        if enable {
            self.service_events
                .try_send(UpdateMonitorEvents::Enable)
                .unwrap();
        } else {
            self.service_events
                .try_send(UpdateMonitorEvents::Disable)
                .unwrap();
        }
    }
}

#[async_trait]
impl Service for UpdateMonitorService {
    fn name(&self) -> &'static str {
        "update-monitor"
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let _application_events_sender = self.application_events.sender.clone();

        let interval = task::interval(Duration::from_secs(UPDATE_POLLING_INTERVAL_SECONDS));
        pin_mut!(interval);

        loop {
            select! {
                _ = interval.next().fuse() => {
                    if !self.is_enabled.load(Ordering::Relaxed) {
                        continue;
                    }

                    let _ = check_version().await;
                },
                msg = this.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            UpdateMonitorEvents::Enable => {
                                if !self.is_enabled.load(Ordering::Relaxed) {
                                    self.is_enabled.store(true, Ordering::Relaxed);
                                    let _ = check_version().await;
                                }
                            }
                            UpdateMonitorEvents::Disable => {
                                self.is_enabled.store(false, Ordering::Relaxed);
                            }
                            UpdateMonitorEvents::Exit => {
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
            .try_send(UpdateMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
