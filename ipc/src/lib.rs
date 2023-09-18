//extern crate kaspa_egui;
//use kaspa_egui::*;
use wasm_bindgen::prelude::*;

//wasm-pack build --target nodejs --out-name ipc --out-dir nodejs/ipc --dev

#[wasm_bindgen]
pub struct TestApi{

}

#[wasm_bindgen]
impl TestApi{
    pub fn send_message(msg: JsValue){

    }
}