use crate::imports::*;
use kaspa_notify::{listener::ListenerId, scope::*};
use kaspa_rpc_core::api::notifications::Notification;
use kaspa_rpc_core::notify::connection::{ChannelConnection, ChannelType};
use kaspa_rpc_core::RpcBlock;

pub enum BlockDagMonitorEvents {
    Exit,
}

pub struct BlockDagMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<BlockDagMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    listener_id: Mutex<Option<ListenerId>>,
    notification_channel: Channel<Notification>,

    pub blocks_by_hash: RwLock<HashMap<kaspa_consensus_core::Hash, Arc<RpcBlock>>>,
    // pub virtual_chain: ,
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
            blocks_by_hash: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_notification_listener(&self) -> Result<()> {
        if let Some(rpc_api) = self.rpc_api() {
            let listener_id = rpc_api.register_new_listener(ChannelConnection::new(
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
}

#[async_trait]
impl Service for BlockDagMonitorService {
    async fn attach_rpc(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        self.rpc_api.lock().unwrap().replace(rpc_api.clone());
        Ok(())
    }

    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.rpc_api.lock().unwrap().take();

        Ok(())
    }

    async fn connect_rpc(self: Arc<Self>) -> Result<()> {
        self.register_notification_listener().await?;
        Ok(())
    }

    async fn disconnect_rpc(self: Arc<Self>) -> Result<()> {
        self.unregister_notification_listener().await?;
        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let _application_events_sender = self.application_events.sender.clone();

        let mut chain: VecDeque<Arc<RpcBlock>> = Default::default();

        loop {
            select! {

                msg = this.notification_channel.receiver.recv().fuse() => {
                    if let Ok(notification) = msg {
                        match notification {
                            Notification::BlockAdded(block_added_notification) => {
                                // println!("BlockAdded: {:?}", block_added_notification);
                                let block = block_added_notification.block.clone();
                                // let daa_score = block.header.daa_score;
                                while chain.len() > 255 {
                                    let block = chain.pop_front().unwrap();
                                    this.blocks_by_hash.write().unwrap().remove(&block.header.hash);
                                }
                                chain.push_back(block.clone());
                                this.blocks_by_hash.write().unwrap().insert(block.header.hash, block);

                            },
                            Notification::VirtualChainChanged(_virtual_chain_changed_notification) => {
                                // println!("VirtualChainChanged: {:?}", virtual_chain_changed_notification);
                            },
                            _ => {
                                println!("notification: {:?}", notification);
                            }
                        }

                        crate::runtime::try_runtime().map(|runtime| runtime.request_repaint());
                    } else {
                        break;
                    }
                },

                msg = this.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            BlockDagMonitorEvents::Exit => {
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
            .try_send(BlockDagMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
