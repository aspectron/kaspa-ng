// use workflow_wasm::extensions::ObjectExtension;

use crate::imports::*;
use kaspa_ng_core::interop;
use kaspa_ng_core::interop::transport::Target;

pub type ListenerClosure = Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>;

#[derive(Default, Clone)]
pub struct ClientSender {}

unsafe impl Send for ClientSender {}
unsafe impl Sync for ClientSender {}

impl ClientSender {
    async fn send_message(&self, target: Target, op: u64, data: Vec<u8>) -> Result<Vec<u8>> {
        // let msg = req_to_jsv(Target::Wallet, 0, &vec![]);
        // send_message(&msg).await

        let (tx, rx) = oneshot::<Result<Vec<u8>>>();
        spawn_local(async move {
            // match send_message(&req_to_jsv(Target::Wallet, op, &data)).await {
            match send_message(&req_to_jsv(target, op, &data)).await {
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
    async fn send_message(&self, target: Target, op: u64, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.send_message(target, op, data).await?)
    }
}

#[async_trait]
impl BorshCodec for ClientSender {
    async fn call(&self, op: u64, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.send_message(Target::Wallet, op, data).await?)
    }
}

#[repr(u8)]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
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
    op: u64,
    data: Vec<u8>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct ServerMessage {
    target: Target,
    kind: ServerMessageKind,
    // op : Option<u64>,
    data: Vec<u8>,
}

pub fn req_to_jsv(target: Target, op: u64, src: &[u8]) -> JsValue {
    let request = ClientMessage {
        target,
        op,
        data: src.to_vec(),
    };

    let data = request.try_to_vec().unwrap();

    JsValue::from(data.to_hex())
}

pub fn jsv_to_req(src: JsValue) -> Result<(Target, u64, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;
    if src.len() < 10 {
        return Err(Error::custom("invalid message length"));
    }

    let request = ClientMessage::try_from_slice(&src).unwrap();
    Ok((request.target, request.op, request.data))
}

pub fn resp_to_jsv(target: Target, response: Result<Vec<u8>>) -> JsValue {
    match response {
        Ok(src) => {
            let response = ServerMessage {
                target,
                kind: ServerMessageKind::Success,
                data: src,
            };

            let data = response.try_to_vec().unwrap();

            JsValue::from(data.to_hex())
        }
        Err(error) => {
            let response = ServerMessage {
                target,
                kind: ServerMessageKind::Error,
                data: error.to_string().as_bytes().to_vec(),
            };

            let data = response.try_to_vec().unwrap();
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
        return Err(Error::custom("invalid message length"));
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

    let data = notify.try_to_vec().unwrap();
    JsValue::from(data.to_hex())
}

pub fn jsv_to_notify(src: JsValue) -> Result<(interop::Target, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;

    let notify = ServerMessage::try_from_slice(&src).unwrap();
    log_info!("### NOTIFICATION MESSAGE: {:?}", notify);
    match notify.kind {
        // ServerMessageKind::Notification => Ok((notify.op.unwrap(), notify.data)),
        ServerMessageKind::Notification => Ok((notify.target, notify.data)),
        _ => Err(Error::custom(
            "Error: jsv_to_notify trying to parse a non-notification message",
        )),
    }
}
