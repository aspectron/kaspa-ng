use crate::imports::*;

// #[derive(Default)]
pub struct ClientTransport {
    application_events: ApplicationEventsChannel,
    closure: Mutex<Option<Rc<ListenerClosure>>>,
    chrome_extension_id: String,
}

unsafe impl Send for ClientTransport {}
unsafe impl Sync for ClientTransport {}

#[async_trait]
impl BorshCodec for ClientTransport {
    async fn call(&self, op: u64, data: Vec<u8>) -> Result<Vec<u8>> {
        let (tx, rx) = oneshot::<Result<Vec<u8>>>();
        spawn_local(async move {
            match send_message(&req_to_jsv(Target::Wallet, op, &data)).await {
                Ok(jsv) => {
                    let resp = jsv_to_resp(&jsv);
                    tx.send(resp).await.unwrap();
                }
                Err(err) => {
                    log_error!("error sending message: {err:?}");
                    tx.send(Err(err.into())).await.unwrap();
                }
            };
        });
        rx.recv()
            .await
            .map_err(|_| Error::custom("Client transport receive channel error"))?
    }
}

impl ClientTransport {
    pub fn new(application_events: ApplicationEventsChannel) -> Self {
        Self {
            application_events,
            chrome_extension_id: runtime_id().unwrap(),
            closure: Mutex::new(None),
        }
    }

    pub fn start(self: &Arc<Self>) {
        // self.runtime.start();
        self.register_listener();
    }

    fn register_listener(self: &Arc<Self>) {
        let this = self.clone();

        let closure = Rc::new(Closure::new(
            // move |msg, sender: Sender, callback: JsValue| -> JsValue {
            move |msg, sender: Sender, _callback: JsValue| -> JsValue {
                // let callback = js_sys::Function::from(callback);
                log_info!("CLIENT RECEIVED MESSAGE: {:?}", msg);
                if let Err(err) = this.handle_notification(msg, sender) {
                    log_error!("notification handling error: {:?}", err);
                }
                JsValue::from(false)
            },
        ));

        log_info!("CLIENT REGISTERING LISTENER...");
        chrome_runtime_on_message::add_listener(closure.clone().as_ref());
        *self.closure.lock().unwrap() = Some(closure);
    }

    fn handle_notification(
        self: &Arc<Self>,
        msg: JsValue,
        sender: Sender,
        // callback: Function,
    ) -> Result<()> {
        log_info!("CLIENT HANDLING NOTIFICATION...");
        if let Some(id) = sender.id() {
            if id != self.chrome_extension_id {
                return Err(Error::custom(
                    "Unknown sender id - foreign requests are forbidden",
                ));
            }
        } else {
            return Err(Error::custom("Sender is missing id"));
        }

        log_info!(
            "[WASM] notification: {:?}, sender id:{:?}",
            msg,
            sender.id(),
            // callback
        );

        let (target, data) = jsv_to_notify(msg)?;

        match target {
            Target::Wallet => {
                let event = Box::new(Events::try_from_slice(&data)?);

                self.application_events
                    .sender
                    .try_send(kaspa_ng_core::events::Events::Wallet { event })
                    .unwrap();

                // spawn_local(async move {

                //     let resp = resp_to_jsv(self.wallet_server.call_with_borsh(op, &data).await);
                //     if let Err(err) = callback.call1(&JsValue::UNDEFINED, &resp) {
                //         log_error!("onMessage callback error: {:?}", err);
                //     }
                // });
            }
            Target::Runtime => {
                todo!()
            }
        }

        Ok(())
    }
}
