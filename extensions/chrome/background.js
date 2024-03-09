import {apiBuilder} from "./api.js";

// chrome.runtime.onMessage.addListener(async (message, sender, reply)=>{
//     console.log("message:", message)
//     console.log("sender:", sender)
//     if (sender.id != chrome.runtime.id)
//         return

//     let {id, op, data} = message;
//     if (op == "inject-page-script"){
//         chrome.scripting.executeScript({
//             args: message.args||[],
//             target: {tabId: sender.tab.id},
//             world: "MAIN",
//             func: apiBuilder
//         });
//     }

//     // if (op == "sign-transaction"){
//     //     reply({id, error:"TODO"})
//     // }

//     // if (op == "connect"){
//     //     reply({id, })
//     // }

//     // if (message.target !== "background")
//     //     return
    
//     // console.log("message, sender")
//     // console.log(message, sender, reply)
//     // reply("from background");
// });

import init from '/kaspa-ng.js';
(async () => {

    function initPageScript(tabId, args){
        chrome.scripting.executeScript({
            args: args||[],
            target: {tabId},
            world: "MAIN",
            func: apiBuilder
        });
    }

    //TODO: move to rust
    async function openPopup(){
        if(chrome.action?.openPopup){
            chrome.action.openPopup();
        }else{
            let win = await chrome.windows.getCurrent();
            let width = 400;
            let left = Math.max(0, win.left + win.width - width);
            chrome.windows.create({url:"popup.html", focused:true, left, width, height:600, type:"panel"})
        }
    }

    globalThis.initPageScript = initPageScript;
    globalThis.openPopup = openPopup;

    let kaspa_ng = await init('/kaspa-ng_bg.wasm');
console.log("init", init);
console.log("kaspa_ng", kaspa_ng);

    // //TODO: move to rust
    // chrome.runtime.onConnect.addListener(function(port) {
    //     console.log("BG: runtime.onConnect:port", port);
    //     if (port.sender.id != chrome.runtime.id)
    //         return

    //     let type = port.name || "CONTENT";
    //     port.onMessage.addListener(async function(msg) {
    //         console.log("BG: msg", msg)

    //         let response = await kaspa_ng.handle_port_message(type, msg);
    //         if (response){
    //             port.postMessage(response);
    //         }
    //     });
    // });
    await kaspa_ng.kaspa_ng_background();
})();
