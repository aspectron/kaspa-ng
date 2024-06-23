use crate::imports::*;
use kaspa_notify::{listener::ListenerId, scope::*};
use kaspa_rpc_core::api::notifications::Notification;
use kaspa_rpc_core::notify::connection::{ChannelConnection, ChannelType};
use kaspa_rpc_core::{RpcBlock, VirtualChainChangedNotification};

pub enum BlockDagMonitorEvents {
    Enable,
    Disable,
    Settings(Arc<BlockDagGraphSettings>),
    Reset,
    Exit,
}

pub struct BlockDagMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<BlockDagMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    listener_id: Mutex<Option<ListenerId>>,
    notification_channel: Channel<Notification>,
    is_enabled: Arc<AtomicBool>,
    is_active: Arc<AtomicBool>,
    is_connected: Arc<AtomicBool>,
    pub chain: Mutex<AHashMap<u64, DaaBucket>>,
    pub separators: Mutex<Vec<u64>>,
    pub new_blocks: Arc<Mutex<AHashSet<kaspa_consensus_core::Hash>>>,
    pub settings: Mutex<Arc<BlockDagGraphSettings>>,
}

impl BlockDagMonitorService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            rpc_api: Mutex::new(None),
            listener_id: Mutex::new(None),
            notification_channel: Channel::<Notification>::unbounded(),
            chain: Mutex::new(AHashMap::new()),
            separators: Mutex::new(Vec::new()),
            new_blocks: Arc::new(Mutex::new(AHashSet::new())),
            is_enabled: Arc::new(AtomicBool::new(false)),
            is_active: Arc::new(AtomicBool::new(false)),
            is_connected: Arc::new(AtomicBool::new(false)),
            settings: Mutex::new(Arc::new(BlockDagGraphSettings::default())),
        }
    }

    pub async fn register_notification_listener(&self) -> Result<()> {
        if let Some(rpc_api) = self.rpc_api() {
            let listener_id = rpc_api.register_new_listener(ChannelConnection::new(
                "blockdag-monitor",
                self.notification_channel.sender.clone(),
                ChannelType::Persistent,
            ));
            *self.listener_id.lock().unwrap() = Some(listener_id);
            rpc_api
                .start_notify(listener_id, Scope::BlockAdded(BlockAddedScope {}))
                .await?;
            rpc_api
                .start_notify(
                    listener_id,
                    Scope::VirtualChainChanged(VirtualChainChangedScope {
                        include_accepted_transaction_ids: false,
                    }),
                )
                .await?;
        }

        Ok(())
    }

    pub async fn unregister_notification_listener(&self) -> Result<()> {
        if let Some(rpc_api) = self.rpc_api() {
            let listener_id = self.listener_id.lock().unwrap().take();
            if let Some(id) = listener_id {
                // we do not need this as we are unregister the entire listener here...
                rpc_api.unregister_listener(id).await?;
            }
        }
        Ok(())
    }

    pub fn rpc_api(&self) -> Option<Arc<dyn RpcApi>> {
        self.rpc_api.lock().unwrap().clone()
    }

    pub fn activate(&self, state: bool) {
        self.is_active.store(state, Ordering::Relaxed);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled.load(Ordering::Relaxed)
    }

    pub fn enable(&self, _current_daa_score: Option<u64>) {
        self.service_events
            .sender
            .try_send(BlockDagMonitorEvents::Enable)
            .unwrap();
        // if let Some(current_daa_score) = current_daa_score {
        //     self._update_separators(current_daa_score);
        // }
    }

    pub fn disable(&self, _current_daa_score: Option<u64>) {
        self.service_events
            .sender
            .try_send(BlockDagMonitorEvents::Disable)
            .unwrap();
        // if let Some(current_daa_score) = current_daa_score {
        //     self._update_separators(current_daa_score);
        // }
    }

    fn _update_separators(&self, current_daa_score: u64) {
        let graph_length_daa = self.settings.lock().unwrap().graph_length_daa as u64;
        let mut separators = self.separators.lock().unwrap();
        while let Some(first) = separators.first() {
            if *first < (current_daa_score - graph_length_daa) {
                separators.remove(0);
            } else {
                break;
            }
        }
        separators.push(current_daa_score);
    }

    pub fn separators(&self) -> MutexGuard<'_, Vec<u64>> {
        self.separators.lock().unwrap()
    }

    pub fn update_settings(&self, settings: BlockDagGraphSettings) {
        self.service_events
            .sender
            .try_send(BlockDagMonitorEvents::Settings(Arc::new(settings)))
            .unwrap();
    }

    pub fn new_blocks(&self) -> MutexGuard<'_, AHashSet<kaspa_consensus_core::Hash>> {
        self.new_blocks.lock().unwrap()
    }

    fn update_new_blocks(&self, block: &Arc<RpcBlock>) {
        let mut new_blocks = self.new_blocks.lock().unwrap();
        new_blocks.insert(block.header.hash);
        let parents = block
            .header
            .parents_by_level
            .iter()
            .flatten()
            .collect::<Vec<_>>();
        parents.iter().for_each(|hash| {
            new_blocks.remove(hash);
        });
    }
}

#[async_trait]
impl Service for BlockDagMonitorService {
    fn name(&self) -> &'static str {
        "block-dag-monitor"
    }

    async fn attach_rpc(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        self.rpc_api.lock().unwrap().replace(rpc_api.clone());
        Ok(())
    }

    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.rpc_api.lock().unwrap().take();

        Ok(())
    }

    async fn connect_rpc(self: Arc<Self>) -> Result<()> {
        self.is_connected.store(true, Ordering::Relaxed);
        if self.is_enabled.load(Ordering::Relaxed) {
            self.register_notification_listener().await?;
        }

        Ok(())
    }

    async fn disconnect_rpc(self: Arc<Self>) -> Result<()> {
        self.is_connected.store(false, Ordering::Relaxed);
        if self.listener_id.lock().unwrap().is_some() {
            self.unregister_notification_listener().await?;
        }

        self.service_events
            .sender
            .try_send(BlockDagMonitorEvents::Reset)
            .unwrap();

        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let _application_events_sender = self.application_events.sender.clone();

        let mut blocks_by_hash: AHashMap<kaspa_consensus_core::Hash, Arc<RpcBlock>> =
            AHashMap::default();

        let mut settings = (*self.settings.lock().unwrap()).clone();
        loop {
            select! {

                msg = self.notification_channel.receiver.recv().fuse() => {
                    if let Ok(notification) = msg {
                        match notification {
                            Notification::BlockAdded(block_added_notification) => {
                                let block = block_added_notification.block.clone();

                                self.update_new_blocks(&block);

                                blocks_by_hash.insert(block.header.hash, block.clone());

                                let daa_score = block.header.daa_score;
                                let mut chain = self.chain.lock().unwrap();
                                if let Some(bucket) = chain.get_mut(&daa_score) {
                                    bucket.push(DagBlock::new(block, &settings), &settings);
                                } else {
                                    let mut bucket = DaaBucket::new(daa_score as f64, DagBlock::new(block, &settings));
                                    bucket.update(&settings);
                                    chain.insert(daa_score, bucket);
                                }

                                let last_daa = daa_score - settings.graph_length_daa as u64;
                                chain.retain(|daa_score, bucket| {
                                    if *daa_score > last_daa {
                                        true
                                    } else {
                                        bucket.blocks.iter().for_each(|block| {
                                            blocks_by_hash.remove(&block.data.header.hash);
                                        });
                                        false
                                    }
                                });
                            },
                            Notification::VirtualChainChanged(virtual_chain_changed_notification) => {
                                let VirtualChainChangedNotification {
                                    removed_chain_block_hashes,
                                    added_chain_block_hashes,
                                    ..
                                } = virtual_chain_changed_notification;

                                removed_chain_block_hashes.iter().for_each(|hash| {
                                    if let Some(block) = blocks_by_hash.get(hash) {
                                        let daa_score = block.header.daa_score;
                                        let mut chain = self.chain.lock().unwrap();
                                        if let Some(bucket) = chain.get_mut(&daa_score) {
                                            bucket.update_vspc(*hash, false, &settings);
                                        }
                                    }
                                });
                                added_chain_block_hashes.iter().for_each(|hash| {
                                    if let Some(block) = blocks_by_hash.get(hash) {
                                        let daa_score = block.header.daa_score;
                                        let mut chain = self.chain.lock().unwrap();
                                        if let Some(bucket) = chain.get_mut(&daa_score) {
                                            bucket.update_vspc(*hash, true, &settings);
                                        }
                                    }
                                });
                                // println!("VirtualChainChanged: {:?}", virtual_chain_changed_notification);
                            },
                            _ => {
                                // println!("notification: {:?}", notification);
                            }
                        }

                        runtime().request_repaint();
                    } else {
                        break;
                    }
                },

                msg = self.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            BlockDagMonitorEvents::Enable => {
                                if !self.is_enabled.load(Ordering::Relaxed) {
                                    self.is_enabled.store(true, Ordering::Relaxed);
                                    if self.rpc_api().is_some() && self.is_connected.load(Ordering::SeqCst) {
                                        self.register_notification_listener().await.unwrap();
                                    }
                                }
                            }
                            BlockDagMonitorEvents::Disable => {
                                if self.is_enabled.load(Ordering::Relaxed) {
                                    self.is_enabled.store(false, Ordering::Relaxed);
                                    self.unregister_notification_listener().await.unwrap();
                                }
                            }
                            BlockDagMonitorEvents::Exit => {
                                if self.is_enabled.load(Ordering::Relaxed) {
                                    self.is_enabled.store(false, Ordering::Relaxed);
                                    self.unregister_notification_listener().await.unwrap();
                                }

                                break;
                            }
                            BlockDagMonitorEvents::Reset => {
                                self.chain.lock().unwrap().clear();
                                blocks_by_hash.clear();
                            }
                            BlockDagMonitorEvents::Settings(new_settings) => {
                                *self.settings.lock().unwrap() = new_settings.clone();
                                settings = new_settings;
                                let mut chain = self.chain.lock().unwrap();
                                for bucket in chain.values_mut() {
                                    bucket.reset(&settings);
                                }
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
            .try_send(BlockDagMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
