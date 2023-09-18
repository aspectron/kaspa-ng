import wasm from "../nodejs/ipc/ipc.js";
console.log("wasm", wasm)
import {Chrome} from "./api.mjs";

let chrome1 = new Chrome();
let chrome2 = new Chrome();


chrome1.runtime.onMessage.addListener((msg, sender, callback)=>{
    console.log("chrome1 onMessage:", msg, sender, callback)
    if (callback){
        callback("got msg in chrome 1:"+msg)
    }
})

chrome2.runtime.onMessage.addListener((msg, sender, callback)=>{
    console.log("chrome2 onMessage:", msg, sender, callback)
    if (callback){
        callback("got msg in chrome 2:"+msg)
    }
})

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

// setTimeout(()=>{

// }, 5000)