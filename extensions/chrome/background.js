import {apiBuilder} from "./api.js";

chrome.runtime.onMessage.addListener(async (message, sender, reply)=>{
    console.log("message:", message)
    console.log("sender:", sender)
    if (sender.id != chrome.runtime.id)
        return

    let {id, op, data} = message;
    if (op == "inject-page-script"){
        chrome.scripting.executeScript({
            args: message.args||[],
            target: {tabId: sender.tab.id},
            world: "MAIN",
            func: apiBuilder
        });
    }

    // if (op == "sign-transaction"){
    //     reply({id, error:"TODO"})
    // }

    // if (op == "connect"){
    //     reply({id, })
    // }

    // if (message.target !== "background")
    //     return
    
    // console.log("message, sender")
    // console.log(message, sender, reply)
    // reply("from background");
});

import init from '/kaspa-ng.js';

(async () => {
    let kaspa_ng = await init('/kaspa-ng_bg.wasm');
console.log("init", init);
console.log("kaspa_ng", kaspa_ng);
    // const wasm = await kaspa.default('./kaspa-wallet/kaspa-wallet_bg.wasm');
    await kaspa_ng.kaspa_ng_background();
})();
