use js_sys::Object;
use kaspa_ng_macros::*;
use kaspa_utils::hex::*;
use kaspa_wallet_core::encryption::sha256_hash;
use kaspa_wallet_core::error::Error;
use kaspa_wallet_core::result::Result;
use rand::Rng;
use wasm_bindgen::prelude::*;

// const SUCCESS: u8 = 0;
// const ERROR: u8 = 1;

#[repr(u8)]
pub enum Target {
    Wallet = 0,
    Interop = 1,
}

impl TryFrom<u8> for Target {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Target::Wallet),
            1 => Ok(Target::Interop),
            _ => Err(Error::custom("invalid message target")),
        }
    }
}

#[repr(u8)]
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

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(extends = Object)]
    #[derive(Debug)]
    pub type Sender;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Sender) -> Option<String>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "runtime"], js_name="sendMessage")]
    pub async fn send_message(s: &JsValue) -> std::result::Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["chrome", "runtime", "onMessage"], js_name="addListener")]
    pub fn add_listener(closure: &Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>);
}

fn chrome() -> std::result::Result<JsValue, JsValue> {
    js_sys::Reflect::get(&js_sys::global(), &"chrome".into())
}

fn runtime() -> std::result::Result<JsValue, JsValue> {
    js_sys::Reflect::get(&chrome()?, &"runtime".into())
}

pub fn runtime_id() -> std::result::Result<String, JsValue> {
    Ok(js_sys::Reflect::get(&runtime()?, &"id".into())?
        .as_string()
        .unwrap())
}

fn mask_data() -> &'static [u8; 128] {
    static mut MASK: Option<Box<[u8; 128]>> = None;
    unsafe {
        MASK.get_or_insert_with(|| {
            let mut data = mask!().to_vec();
            data.extend(
                runtime_id()
                    .expect("missing runtime id")
                    .as_bytes()
                    .to_vec(),
            );
            data = sha256_hash(&data).as_ref().to_vec();
            (0..3).for_each(|_| {
                data.extend(sha256_hash(&data).as_ref());
            });
            Box::new(data.try_into().unwrap())
        })
    }
}

fn mask(data: &mut [u8], src: &[u8], index: &mut usize, mask: &[u8]) {
    for i in 0..src.len() {
        data[i] = src[i] ^ mask[*index];
        *index += 1;
        if *index == mask.len() {
            *index = 0;
        }
    }
}

pub fn req_to_jsv(target : Target, op: u32, src: &[u8]) -> JsValue {
    let mask_data = mask_data();
    let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
    let mut data = vec![0; src.len() + 5];
    data[0] = index as u8;
    data[1] = target as u8 ^ mask_data[index];
    index += 1;
    // mask(&mut data[1..1], &[target as u8], &mut index, mask_data);
    mask(
        &mut data[2..6],
        op.to_le_bytes().as_ref(),
        &mut index,
        mask_data,
    );
    mask(&mut data[6..], src, &mut index, mask_data);
    JsValue::from(data.to_hex())
}

pub fn jsv_to_req(src: JsValue) -> Result<(Target, u32, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;
    if src.len() < 6 {
        return Err(Error::custom("invalid message length"));
    }
    let mask_data = mask_data();
    let mut index = src[0] as usize;
    let mut data = vec![0; src.len() - 1];
    mask(&mut data, &src[1..], &mut index, mask_data);
    let target = Target::try_from(data[0])?;
    let op = u32::from_le_bytes(data[1..5].try_into().unwrap());
    Ok((target, op, data[5..].to_vec()))
}

pub fn resp_to_jsv(response: Result<Vec<u8>>) -> JsValue {
    let mask_data = mask_data();
    let mut index = rand::thread_rng().gen::<usize>() % (mask_data.len() - 1);

    match response {
        Ok(src) => {
            let mut data = vec![0; src.len() + 2];
            data[0] = index as u8;
            data[1] = ServerMessageKind::Success as u8 ^ mask_data[index];
            index += 1;
            mask(&mut data[2..], &src, &mut index, mask_data);
            JsValue::from(data.to_hex())
        }
        Err(error) => {
            let error = error.to_string();
            let src = error.as_bytes();
            let mut data = vec![0; src.len() + 2];
            data[0] = index as u8;
            data[1] = ServerMessageKind::Error as u8 ^ mask_data[index];
            index += 1;
            mask(&mut data[2..], src, &mut index, mask_data);
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

    let mask_data = mask_data();
    let mut index = src[0] as usize;
    let mut data = vec![0; src.len() - 1];
    mask(&mut data, &src[1..], &mut index, mask_data);

    let kind = ServerMessageKind::try_from(data[0])?;
    match kind {
        ServerMessageKind::Success => Ok(data[1..].to_vec()),
        ServerMessageKind::Error => {
            let error = String::from_utf8(data[1..].to_vec())?;
            Err(Error::custom(error))
        }
        _ => Err(Error::custom("invalid response code")),
    }
}

// ----


pub fn notify_to_jsv(op: u32, src: &[u8]) -> JsValue {
    let mask_data = mask_data();
    let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
    let mut data = vec![0; src.len() + 5];
    data[0] = index as u8;
    mask(
        &mut data[1..5],
        op.to_le_bytes().as_ref(),
        &mut index,
        mask_data,
    );
    mask(&mut data[5..], src, &mut index, mask_data);
    JsValue::from(data.to_hex())
}

pub fn jsv_to_notify(src: JsValue) -> Result<(u32, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;
    if src.len() < 5 {
        return Err(Error::custom("invalid message length"));
    }
    let mask_data = mask_data();
    let mut index = src[0] as usize;
    let mut data = vec![0; src.len() - 1];
    mask(&mut data, &src[1..], &mut index, mask_data);
    let op = u32::from_le_bytes(data[0..4].try_into().unwrap());
    Ok((op, data[4..].to_vec()))
}

