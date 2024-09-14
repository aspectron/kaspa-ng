use crate::imports::*;
use kaspa_ng_core::interop;
use kaspa_ng_core::interop::transport::Target;

pub type ListenerClosure = Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>;

#[derive(Default, Clone)]
pub struct ClientSender {}

unsafe impl Send for ClientSender {}
unsafe impl Sync for ClientSender {}

impl ClientSender {
    async fn send_message(&self, target: Target, data: Vec<u8>) -> Result<Vec<u8>> {
        let (tx, rx) = oneshot::<Result<Vec<u8>>>();
        spawn_local(async move {
            match send_message(&req_to_jsv(target, &data)).await {
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
#[async_trait]
impl interop::Sender for ClientSender {
    async fn send_message(&self, target: Target, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.send_message(target, data).await?)
    }
}

#[async_trait]
impl BorshCodec for ClientSender {
    async fn call(&self, op: u64, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self
            .send_message(
                Target::Wallet,
                borsh::to_vec(&WalletMessage::new(op, data)).unwrap(),
            )
            .await?)
    }
}

#[repr(u8)]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
enum ServerMessageKind {
    Success = 0,
    Error = 1,
    Notification = 2,
}

impl TryFrom<u8> for ServerMessageKind {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(ServerMessageKind::Success),
            1 => Ok(ServerMessageKind::Error),
            2 => Ok(ServerMessageKind::Notification),
            _ => Err(Error::custom("invalid server message kind")),
        }
    }
}

// fn mask_data() -> &'static [u8; 128] {
//     static mut MASK: Option<Box<[u8; 128]>> = None;
//     unsafe {
//         MASK.get_or_insert_with(|| {
//             let mut data = crnd!().to_vec();
//             data.extend(
//                 runtime_id()
//                     .expect("missing runtime id")
//                     .as_bytes()
//                     .to_vec(),
//             );
//             data = sha256_hash(&data).as_ref().to_vec();
//             (0..3).for_each(|_| {
//                 data.extend(sha256_hash(&data).as_ref());
//             });
//             Box::new(data.try_into().unwrap())
//         })
//     }
// }

// fn mask(data: &mut [u8], src: &[u8], index: &mut usize, mask: &[u8]) {
//     for i in 0..src.len() {
//         data[i] = src[i] ^ mask[*index];
//         *index += 1;
//         if *index == mask.len() {
//             *index = 0;
//         }
//     }
// }

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientMessage {
    target: Target,
    data: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct WalletMessage {
    pub op: u64,
    pub data: Vec<u8>,
}
impl WalletMessage {
    pub fn new(op: u64, data: Vec<u8>) -> Self {
        Self { op, data }
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct ServerMessage {
    target: Target,
    kind: ServerMessageKind,
    data: Vec<u8>,
}

pub fn req_to_jsv(target: Target, src: &[u8]) -> JsValue {
    let request = ClientMessage {
        target,
        data: src.to_vec(),
    };

    let data = borsh::to_vec(&request).unwrap();

    JsValue::from(data.to_hex())
}

pub fn jsv_to_req(src: JsValue) -> Result<(Target, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;
    if src.len() < 6 {
        return Err(Error::custom(format!(
            "invalid message length: {}",
            src.len()
        )));
    }

    let request = ClientMessage::try_from_slice(&src).unwrap();
    Ok((request.target, request.data))
}

pub fn resp_to_jsv(target: Target, response: Result<Vec<u8>>) -> JsValue {
    match response {
        Ok(src) => {
            let response = ServerMessage {
                target,
                kind: ServerMessageKind::Success,
                data: src,
            };

            let data = borsh::to_vec(&response).unwrap();

            JsValue::from(data.to_hex())
        }
        Err(error) => {
            let response = ServerMessage {
                target,
                kind: ServerMessageKind::Error,
                data: error.to_string().as_bytes().to_vec(),
            };

            let data = borsh::to_vec(&response).unwrap();
            JsValue::from(data.to_hex())
        }
    }
}

pub fn jsv_to_resp(jsv: &JsValue) -> Result<Vec<u8>> {
    let src = Vec::<u8>::from_hex(
        jsv.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;
    if src.len() < 2 {
        return Err(Error::custom(format!(
            "invalid message length: {}",
            src.len()
        )));
    }

    let response = ServerMessage::try_from_slice(&src).unwrap();

    match response.kind {
        ServerMessageKind::Success => Ok(response.data),
        ServerMessageKind::Error => {
            let error = String::from_utf8(response.data)?;
            Err(Error::custom(error))
        }
        _ => Err(Error::custom("invalid response code")),
    }
}

// ----

pub fn notify_to_jsv(target: Target, src: &[u8]) -> JsValue {
    let notify = ServerMessage {
        target,
        kind: ServerMessageKind::Notification,
        data: src.to_vec(),
    };

    let data = borsh::to_vec(&notify).unwrap();
    JsValue::from(data.to_hex())
}

pub fn jsv_to_notify(src: JsValue) -> Result<(interop::Target, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;

    let notify = ServerMessage::try_from_slice(&src).unwrap();
    match notify.kind {
        ServerMessageKind::Notification => Ok((notify.target, notify.data)),
        _ => Err(Error::custom(
            "Error: jsv_to_notify trying to parse a non-notification message",
        )),
    }
}
