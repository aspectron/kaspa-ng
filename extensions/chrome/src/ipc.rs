pub use chrome_sys::prelude::*;
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

pub fn req_to_jsv(target: Target, op: u64, src: &[u8]) -> JsValue {
    let mask_data = mask_data();
    let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
    let mut data = vec![0; src.len() + 2 + 8];
    data[0] = index as u8;
    data[1] = target as u8 ^ mask_data[index];
    index += 1;
    // mask(&mut data[1..1], &[target as u8], &mut index, mask_data);
    mask(
        &mut data[2..10],
        op.to_le_bytes().as_ref(),
        &mut index,
        mask_data,
    );
    mask(&mut data[10..], src, &mut index, mask_data);
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
    let mask_data = mask_data();
    let mut index = src[0] as usize;
    let mut data = vec![0; src.len() - 1];
    mask(&mut data, &src[1..], &mut index, mask_data);
    let target = Target::try_from(data[0])?;
    let op = u64::from_le_bytes(data[1..9].try_into().unwrap());
    Ok((target, op, data[9..].to_vec()))
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

pub fn notify_to_jsv(op: u64, src: &[u8]) -> JsValue {
    let mask_data = mask_data();
    let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
    let mut data = vec![0; src.len() + 5];
    data[0] = index as u8;
    mask(
        &mut data[1..9],
        op.to_le_bytes().as_ref(),
        &mut index,
        mask_data,
    );
    mask(&mut data[9..], src, &mut index, mask_data);
    JsValue::from(data.to_hex())
}

pub fn jsv_to_notify(src: JsValue) -> Result<(u64, Vec<u8>)> {
    let src = Vec::<u8>::from_hex(
        src.as_string()
            .ok_or(Error::custom("expecting string"))?
            .as_str(),
    )?;
    if src.len() < 9 {
        return Err(Error::custom("invalid message length"));
    }
    let mask_data = mask_data();
    let mut index = src[0] as usize;
    let mut data = vec![0; src.len() - 1];
    mask(&mut data, &src[1..], &mut index, mask_data);
    let op = u64::from_le_bytes(data[0..8].try_into().unwrap());
    Ok((op, data[8..].to_vec()))
}
