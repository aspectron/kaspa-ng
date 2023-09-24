use js_sys::Object;
use wasm_bindgen::prelude::*;
// use workflow_log::log_info;
use kaspa_wallet_core::result::Result;
use kaspa_wallet_core::error::Error;
use kaspa_utils::hex::*;
use kaspa_ng_macros::*;
use kaspa_wallet_core::encryption::sha256_hash;
use rand::Rng;

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(extends = Object)]
    #[derive(Debug)]
    pub type Sender;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Sender) -> Option<String>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "runtime"], js_name="sendMessage")]
    pub async fn send_message(s: &JsValue) -> std::result::Result<JsValue,JsValue>;

    // #[wasm_bindgen(js_namespace = ["chrome", "runtime"], getter, js_name="id")]
    // fn runtime_id()->String;

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
    static mut MASK : Option<Box<[u8; 128]>> = None;
    unsafe {
        MASK.get_or_insert_with(|| {
            let mut data = mask!().to_vec();
            data.extend(runtime_id().expect("missing runtime id").as_bytes().to_vec());
            data = sha256_hash(&data).as_ref().to_vec();
            (0..3).for_each(|_|{
                data.extend(sha256_hash(&data).as_ref());
            });

            workflow_log::log_info!("*************** mask_data: {}", data.to_hex());

            Box::new(data.try_into().unwrap())
        })
    }
}

fn mask(data : &mut [u8], src : &[u8], index : &mut usize, mask : &[u8]) {
    // let mask = mask_data();
    for i in 0..src.len() {
        data[i] = src[i] ^ mask[*index];
        *index += 1;
        if *index == mask.len() {
            *index = 0;
        }
    }
}

// fn serialize(op: u32, src : &[u8]) -> JsValue {
//     let mask_data = mask_data();

//     let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
//     let mut data = vec![0;src.len()+5];
//     data[0..4].copy_from_slice(op.to_le_bytes().as_ref());
//     data[5] = index as u8;
//     mask(&mut data[5..], src, index, mask_data);

//     JsValue::from(data.to_hex())
// }

// fn deserialize(src : JsValue) -> Result<(u32,Vec<u8>)> {

//     let src = Vec::<u8>::from_hex(src.as_string().ok_or(Error::custom("expecting string"))?.as_str())?;
//     if src.len() < 5 {
//         return Err(Error::custom("invalid message length"));
//     }    
//     let mask_data = mask_data();
//     let op = u32::from_le_bytes(src[0..4].try_into().unwrap());
//     let mut index = src[5] as usize;
//     let mut data = vec![0;src.len()-5];
//     mask(&mut data, &src[5..], index, mask_data);
//     // for i in 0..data.len() {
//     //     data[i] = src[i+1] ^ mask[index];
//     //     index += 1;
//     //     if index == mask.len() {
//     //         index = 0;
//     //     }
//     // }

//     Ok((op,data))
// }

pub fn req_to_jsv(op: u32, src: &[u8]) -> JsValue {

    let mask_data = mask_data();

    let mut index = rand::thread_rng().gen::<usize>() % mask_data.len();
    let mut data = vec![0;src.len()+5];
    data[0] = index as u8;
    mask(&mut data[1..5], op.to_le_bytes().as_ref(), &mut index, mask_data);
    // data[1..5].copy_from_slice(op.to_le_bytes().as_ref());
    mask(&mut data[5..], src, &mut index, mask_data);
    JsValue::from(data.to_hex())
}

pub fn jsv_to_req(src : JsValue) -> Result<(u32,Vec<u8>)> {
    let src = Vec::<u8>::from_hex(src.as_string().ok_or(Error::custom("expecting string"))?.as_str())?;
    if src.len() < 5 {
        return Err(Error::custom("invalid message length"));
    }    
    let mask_data = mask_data();
    let mut index = src[0] as usize;
    let mut data = vec![0;src.len()-1];
    mask(&mut data, &src[1..], &mut index, mask_data);
    let op = u32::from_le_bytes(data[0..4].try_into().unwrap());
    Ok((op, data[4..].to_vec()))
}

// pub struct Request {
//     pub op: u32,
//     pub data: Vec<u8>,
// }

// impl Request {

//     pub fn new(op: u32, data: Vec<u8>) -> Self {
//         Self {
//             op,
//             data
//             //: data.to_vec()
//         }
//     }

//     pub fn to_jsv(op: u32, src: &[u8]) -> JsValue {

//         let mask_data = mask_data();

//         let index = rand::thread_rng().gen::<usize>() % mask_data.len();
//         let mut data = vec![0;src.len()+5];
//         data[0..4].copy_from_slice(op.to_le_bytes().as_ref());
//         data[4] = index as u8;
//         mask(&mut data[5..], src, index, mask_data);
//         JsValue::from(data.to_hex())
//     }

//     pub fn from_jsv(src : JsValue) -> Result<Self> {
//         let src = Vec::<u8>::from_hex(src.as_string().ok_or(Error::custom("expecting string"))?.as_str())?;
//         if src.len() < 5 {
//             return Err(Error::custom("invalid message length"));
//         }    
//         let mask_data = mask_data();
//         let op = u32::from_le_bytes(src[0..4].try_into().unwrap());
//         let index = src[4] as usize;
//         let mut data = vec![0;src.len()-5];
//         mask(&mut data, &src[5..], index, mask_data);
//         Ok(Self { op, data })
//     }
// }

// impl From<Request> for JsValue {
//     fn from(request: Request) -> Self {
//         Request::to_jsv(request.op, &request.data)
//     }
// }

// impl TryFrom<JsValue> for Request {
//     type Error = Error;

//     fn try_from(src: JsValue) -> Result<Self, Self::Error> {
//         Request::from_jsv(src)
//     }
// }


// pub struct Response {
//     pub result : Result<Vec<u8>>
// }

// impl Response {
//     pub fn new(result : Result<Vec<u8>>) -> Self {
//         Self {
//             result
//         }
//     }

//     // fn to_jsv()

//     fn data_to_jsv(src : &[u8]) -> JsValue {
//         let mask_data = mask_data();

//         let index = rand::thread_rng().gen::<usize>() % mask_data.len();
//         let mut data = vec![0;src.len()+1];
//         data[0] = index as u8;
//         mask(&mut data[1..], src, index, mask_data);
    
//         JsValue::from(data.to_hex())
//     }

//     fn str_to_data(src : &str) -> Result<Vec<u8>> {

//         let src = Vec::<u8>::from_hex(src)?; 
//         if src.len() < 1 {
//             return Err(Error::custom("invalid message length"));
//         }    
//         let mask_data = mask_data();
//         let index = src[0] as usize;
//         let mut data = vec![0;src.len()-1];
//         mask(&mut data, &src[1..], index, mask_data);
//         Ok(data)
//     }
// }

// impl From<Response> for Result<Vec<u8>> {
//     fn from(response: Response) -> Self {
//         response.result
//     }
// }

// impl From<Response> for JsValue {
//     fn from(response: Response) -> Self {
        
//         match response.result {
//             Ok(data) => {
//                 Response::data_to_jsv(&data)
//             }
//             Err(error) => {
//                 let msg = Object::new();
//                 js_sys::Reflect::set(&msg, &"error".into(), &JsValue::from(error.to_string())).unwrap();
//                 msg.into()
//             }
//         }
//     }
// }

// impl TryFrom<JsValue> for Response {
//     type Error = Error;

//     fn try_from(msg: JsValue) -> Result<Self, Self::Error> {

//         if let Some(hex) = msg.as_string() {
//             Ok(Self { result : Response::str_to_data(hex.as_str()) })
//         } else {
//             let error = js_sys::Reflect::get(&msg, &"error".into()).unwrap();
//             if let Some(error) = error.as_string() {
//                 Err(Error::custom(error))
//             } else {
//                 Err(Error::custom("invalid response message: no data or error property"))
//             }
//         }
//     }
// }

const SUCCESS: u8 = 0;
const ERROR: u8 = 1;


pub fn resp_to_jsv(response : Result<Vec<u8>>) -> JsValue {
    let mask_data = mask_data();
    let mut index = rand::thread_rng().gen::<usize>() % (mask_data.len()-1);

    match response {
        Ok(src) => {
            let mut data = vec![0;src.len()+2];
            data[0] = index as u8;
            data[1] = SUCCESS ^ mask_data[index];
            index += 1;
            mask(&mut data[2..], &src, &mut index, mask_data);
            JsValue::from(data.to_hex())
        },
        Err(error) => {
            let error = error.to_string();
            let src = error.as_bytes();
            let mut data = vec![0;src.len()+2];
            data[0] = index as u8;
            data[1] = ERROR ^ mask_data[index];
            index += 1;
            mask(&mut data[2..], &src, &mut index, mask_data);
            JsValue::from(data.to_hex())
        }
    }
}

pub fn jsv_to_resp(jsv : &JsValue) -> Result<Vec<u8>> {

    let src = Vec::<u8>::from_hex(jsv.as_string().ok_or(Error::custom("expecting string"))?.as_str())?;
    // let src = Vec::<u8>::from_hex(src)?; 
    if src.len() < 2 {
        return Err(Error::custom("invalid message length"));
    }    
    let mask_data = mask_data();
    let mut index = src[0] as usize;
    let mut data = vec![0;src.len()-1];
    mask(&mut data, &src[1..], &mut index, mask_data);

    let code = data[0]; //src[1] ^ mask_data[index];
    match code {
        SUCCESS => {
            // let mut data = vec![0;src.len()-2];
            // mask(&mut data, &src[2..], index+1, mask_data);
            Ok(data[1..].to_vec())
        }
        ERROR => {
            // let mut data = vec![0;src.len()-2];
            // mask(&mut data, &src[2..], index+1, mask_data);
            let error = String::from_utf8(data[1..].to_vec())?;
            Err(Error::custom(error))
        }
        _ => {
            Err(Error::custom("invalid response code"))
        }
    }
}


// ------



// pub fn create_request(op : u32, data : &[u8]) -> JsValue {
//     let msg = Object::new();
//     let data = data.to_hex();
//     js_sys::Reflect::set(&msg, &"op".into(), &JsValue::from(op)).unwrap();
//     js_sys::Reflect::set(&msg, &"data".into(), &JsValue::from(data)).unwrap();
//     msg.into()
// }

// pub fn parse_request(msg : JsValue) -> Result<(u32, Vec<u8>)> {
//     let op = js_sys::Reflect::get(&msg, &"op".into()).unwrap();
//     let data = js_sys::Reflect::get(&msg, &"data".into()).unwrap();
//     let op = op.as_f64().expect("no op property") as u32;
//     let data = data.as_string().expect("no data property").from_hex().expect("invalid hex");
//     Ok((op, data))
// }

// #[wasm_bindgen]
// pub struct Ipc {
//     closure: Option<Closure<dyn FnMut(JsValue, Sender, JsValue)>>,
//     runtime_id: Option<String>,
//     // _chrome:JsValue
// }

// // #[wasm_bindgen]
// impl Ipc {
//     // #[wasm_bindgen(constructor)]
//     pub fn new() -> Ipc {
//         // let chrome = chrome().unwrap();
//         // log_info!("chrome: {:?}", chrome);
//         // Construct a new closure.

//         let mut ipc = Self {
//             // _chrome: chrome
//             runtime_id: Some(runtime_id().unwrap()),
//             closure: None,
//         };
//         log_info!("runtime_id: {:?}", ipc.runtime_id);
//         ipc.init();

//         ipc
//     }
//     // #[wasm_bindgen(js_name = "sendMessage")]
//     pub async fn send_message(&self, msg: JsValue) -> Result<JsValue> {
//         Ok(send_message(&msg).await?)
//     }
// }

// impl Ipc {
//     fn init(&mut self) {
//         let runtime_id = self.runtime_id.clone();
//         let closure = Closure::new(move |msg, sender: Sender, callback| {
//             if sender.id() != runtime_id {
//                 log_info!("SAME RUNTIME ID!!!");
//                 return;
//             }
//             log_info!(
//                 "[WASM] msg: {:?}, sender id:{:?}, {:?}",
//                 msg,
//                 sender.id(),
//                 callback
//             );
//         });
//         self.closure = Some(closure);
//         add_listener(self.closure.as_ref().unwrap());
//     }
// }

// #[wasm_bindgen]
// pub fn test_send_message() -> Result<(),String> {

//     let o = Object::new();
//     // let id = runtime_id().unwrap();
//     // js_sys::Reflect::set(&o, &"id".into(), &JsValue::from(id)).unwrap();
//     js_sys::Reflect::set(&o, &"message".into(), &JsValue::from("abc123")).unwrap();

//     send_message(&o.into());
//     // send_message(&JsValue::from_str("test message from WASM"));

//     Ok(())
// }