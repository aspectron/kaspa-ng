console.log("in background")

function init_kaspa_object2(key){
    console.log("pure JS init_kaspa_object2")
}

chrome.runtime.onMessage.addListener(async (message, sender, reply)=>{
    console.log("message, sender")
    console.log(message, sender, reply)
    if (sender.id != chrome.runtime.id)
        return

    if (message?.op == "inject-page-script"){
        console.log("calling kaspa_ng.init_kaspa_object_api", message.args[0], sender.tab.id)
        // @aspect tried here from JS side by passing function from here too
        // but no success
        await kaspa_ng.init_kaspa_object_api(message.args[0], sender.tab.id, init_kaspa_object2);
    }

    // if (message.target !== "background")
    //     return
    
    // console.log("message, sender")
    // console.log(message, sender, reply)
    // reply("from background");
});

// var importObject = { imports: { imported_func: arg => console.log(arg) } };
// WebAssembly.instantiateStreaming(fetch('simple.wasm'), importObject)
// .then(obj => obj.instance.exports.exported_func());//output 42 in console

import init from '/kaspa-ng.js';
let kaspa_ng;
let me = this;
(async () => {
    kaspa_ng = await init('/kaspa-ng_bg.wasm');
    self.kaspa_ng = kaspa_ng;
console.log("init", init);
console.log("kaspa_ng", kaspa_ng);
    // const wasm = await kaspa.default('./kaspa-wallet/kaspa-wallet_bg.wasm');
    await kaspa_ng.kaspa_ng_background();
})();
