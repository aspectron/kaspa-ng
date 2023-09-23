import {TestApi,test_send_message} from "../nodejs/ipc/ipc.js";
console.log("TestApi", TestApi)
import {Chrome} from "./api.mjs";


globalThis.chrome = new Chrome();

let api = new TestApi();

let chrome1 = new Chrome();
let chrome2 = new Chrome();
let external_chrome = new Chrome("external-chrome-id");


chrome1.runtime.onMessage.addListener((msg, sender, callback)=>{
    if (sender.id != chrome.runtime.id)
        return
    console.log("chrome1 onMessage:", msg, sender, callback)
    if (callback){
        callback("got msg in chrome 1:"+msg)
    }
})

chrome2.runtime.onMessage.addListener((msg, sender, callback)=>{
    if (sender.id != chrome.runtime.id)
        return
    console.log("chrome2 onMessage:", msg, sender, callback)
    if (callback){
        callback("got msg in chrome 2:"+msg)
    }
})
/*
chrome1.runtime.sendMessage("without callback")

chrome1.runtime.sendMessage("123", (relpy)=>{
    console.log("chrome1 sendMessage relpy:", relpy);
})

chrome1.runtime.sendMessage("xyz", (relpy)=>{
    console.log("chrome1 sendMessage relpy:", relpy);
})

chrome2.runtime.sendMessage("hello", (relpy)=>{
    console.log("chrome2 sendMessage relpy:", relpy);
})



api.sendMessage("from wasm");
*/

test_send_message();

external_chrome.runtime.sendMessage("should not see this message");

// setTimeout(()=>{

// }, 5000)