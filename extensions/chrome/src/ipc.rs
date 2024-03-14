use workflow_wasm::extensions::ObjectExtension;

use crate::imports::*;

// const SUCCESS: u8 = 0;
// const ERROR: u8 = 1;
pub type ListenerClosure = Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>;

#[repr(u8)]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum Target {
    Wallet = 0,
    Runtime = 1,
}

impl TryFrom<u8> for Target {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Target::Wallet),
            1 => Ok(Target::Runtime),
            _ => Err(Error::custom("invalid message target")),
        }
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

    // let mask_data = mask_data();
    // let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
    // let mut data = vec![0; src.len() + 2 + 8];
    // data[0] = index as u8;
    // data[1] = target as u8 ^ mask_data[index];
    // index += 1;
    // // mask(&mut data[1..1], &[target as u8], &mut index, mask_data);
    // mask(
    //     &mut data[2..10],
    //     op.to_le_bytes().as_ref(),
    //     &mut index,
    //     mask_data,
    // );
    // mask(&mut data[10..], src, &mut index, mask_data);
    // let data =

    let obj = js_sys::Object::new();
    obj.set("type", &"Internal".into()).unwrap();
    obj.set("data", &data.to_hex().into()).unwrap();

    obj.into()
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

    // let mask_data = mask_data();
    // let mut index = src[0] as usize;
    // let mut data = vec![0; src.len() - 1];
    // mask(&mut data, &src[1..], &mut index, mask_data);
    // let target = Target::try_from(data[0])?;
    // let op = u64::from_le_bytes(data[1..9].try_into().unwrap());
    // Ok((target, op, data[9..].to_vec()))
}

pub fn resp_to_jsv(target: Target, response: Result<Vec<u8>>) -> JsValue {
    // let mask_data = mask_data();
    // let mut index = rand::thread_rng().gen::<usize>() % (mask_data.len() - 1);

    match response {
        Ok(src) => {
            let response = ServerMessage {
                target, // : Target::Runtime,
                kind: ServerMessageKind::Success,
                // op : None,
                data: src,
            };

            let data = response.try_to_vec().unwrap();

            // let mut data = vec![0; src.len() + 2];
            // data[0] = index as u8;
            // data[1] = ServerMessageKind::Success as u8 ^ mask_data[index];
            // index += 1;
            // mask(&mut data[2..], &src, &mut index, mask_data);
            JsValue::from(data.to_hex())
        }
        Err(error) => {
            let response = ServerMessage {
                target: Target::Runtime,
                kind: ServerMessageKind::Error,
                // op : None,
                data: error.to_string().as_bytes().to_vec(),
            };

            let data = response.try_to_vec().unwrap();
            // let error = error.to_string();
            // let src = error.as_bytes();
            // let mut data = vec![0; src.len() + 2];
            // data[0] = index as u8;
            // data[1] = ServerMessageKind::Error as u8 ^ mask_data[index];
            // index += 1;
            // mask(&mut data[2..], src, &mut index, mask_data);
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

    // let mask_data = mask_data();
    // let mut index = src[0] as usize;
    // let mut data = vec![0; src.len() - 1];
    // mask(&mut data, &src[1..], &mut index, mask_data);

    // let kind = ServerMessageKind::try_from(data[0])?;
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

// pub fn notify_to_jsv(op: u64, src: &[u8]) -> JsValue {
pub fn notify_to_jsv(target: Target, src: &[u8]) -> JsValue {
    // let notify =

    let notify = ServerMessage {
        target, //: Target::Runtime,
        kind: ServerMessageKind::Notification,
        // op : Some(op),
        data: src.to_vec(),
    };

    let data = notify.try_to_vec().unwrap();
    // }

    // let mask_data = mask_data();
    // let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
    // let mut data = vec![0; src.len() + 5];
    // data[0] = index as u8;
    // mask(
    //     &mut data[1..9],
    //     op.to_le_bytes().as_ref(),
    //     &mut index,
    //     mask_data,
    // );
    // mask(&mut data[9..], src, &mut index, mask_data);
    JsValue::from(data.to_hex())
}

// pub fn jsv_to_notify(src: JsValue) -> Result<(u64, Vec<u8>)> {
pub fn jsv_to_notify(src: JsValue) -> Result<(Target, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;
    // if src.len() < 9 {
    //     return Err(Error::custom("invalid message length"));
    // }

    let notify = ServerMessage::try_from_slice(&src).unwrap();
    log_info!("### NOTIFICATION MESSAGE: {:?}", notify);
    match notify.kind {
        // ServerMessageKind::Notification => Ok((notify.op.unwrap(), notify.data)),
        ServerMessageKind::Notification => Ok((notify.target, notify.data)),
        _ => Err(Error::custom(
            "Error: jsv_to_notify trying to parse a non-notification message",
        )),
        // _ => Err(Error::custom("invalid notification code")),
    }

    // let mask_data = mask_data();
    // let mut index = src[0] as usize;
    // let mut data = vec![0; src.len() - 1];
    // mask(&mut data, &src[1..], &mut index, mask_data);
    // let op = u64::from_le_bytes(data[0..8].try_into().unwrap());
    // Ok((op, data[8..].to_vec()))
}
