//extern crate kaspa_egui;
//use kaspa_egui::*;
use wasm_bindgen::prelude::*;
use workflow_log::log_info;
use js_sys::Object;

//wasm-pack build --target nodejs --out-name ipc --out-dir nodejs/ipc --dev

// #[wasm_bindgen]
// #[derive(Debug)]
// pub struct Sender {
//     pub id: Option<String>,
// }

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(extends = Object)]
    #[derive(Debug)]
    type Sender;

    #[wasm_bindgen(method, getter)]
    fn id(this: &Sender)->Option<String>;


    #[wasm_bindgen(js_namespace = ["chrome", "runtime"], js_name="sendMessage")]
    fn send_message(s: &JsValue);

    // #[wasm_bindgen(js_namespace = ["chrome", "runtime"], getter, js_name="id")]
    // fn runtime_id()->String;

    #[wasm_bindgen(js_namespace = ["chrome", "runtime", "onMessage"], js_name="addListener")]
    fn add_listener(closure: &Closure<dyn FnMut(JsValue, Sender, JsValue)>);
    
}


fn chrome() -> std::result::Result<JsValue, JsValue> {
    js_sys::Reflect::get(&js_sys::global(), &"chrome".into())
}

fn runtime() -> std::result::Result<JsValue, JsValue> {
    js_sys::Reflect::get(&chrome()?, &"runtime".into())
}

fn runtime_id()->std::result::Result<String, JsValue>{
    Ok(js_sys::Reflect::get(&runtime()?, &"id".into())?.as_string().unwrap())
}


#[wasm_bindgen]
pub struct TestApi{
    closure: Option<Closure<dyn FnMut(JsValue, Sender, JsValue)>>,
    runtime_id: Option<String>,
    // _chrome:JsValue
}

#[wasm_bindgen]
impl TestApi{
    #[wasm_bindgen(constructor)]
    pub fn new()->TestApi{
        // let chrome = chrome().unwrap();
        // log_info!("chrome: {:?}", chrome);
        // Construct a new closure.

        let mut api = Self{
            // _chrome: chrome
            runtime_id: Some(runtime_id().unwrap()),
            closure: None
        };
log_info!("runtime_id: {:?}", api.runtime_id);
        api.init();

        api
    }
    #[wasm_bindgen(js_name="sendMessage")]
    pub fn send_message(&self, msg: JsValue){
        send_message(&msg)
    }
}

impl TestApi{
    fn init(&mut self){
        let runtime_id = self.runtime_id.clone();
        let closure = Closure::new(move |msg, sender:Sender, callback|{
            if sender.id() != runtime_id{
                log_info!("SAME RUNTIME ID!!!");
                return
            }
            log_info!("[WASM] msg: {:?}, sender id:{:?}, {:?}", msg, sender.id(), callback);
        });
        self.closure = Some(closure);
        add_listener(self.closure.as_ref().unwrap());
    }
}

#[wasm_bindgen]
pub fn test_send_message() -> Result<(),String> {

    let o = Object::new();
    // let id = runtime_id().unwrap();
    // js_sys::Reflect::set(&o, &"id".into(), &JsValue::from(id)).unwrap();
    js_sys::Reflect::set(&o, &"message".into(), &JsValue::from("abc123")).unwrap();

    send_message(&o.into());
    // send_message(&JsValue::from_str("test message from WASM"));

    Ok(())
}

