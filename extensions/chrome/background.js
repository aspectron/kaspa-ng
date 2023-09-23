chrome.runtime.onMessage.addListener((message, sender, reply)=>{
    if (sender.id != chrome.runtime.id)
        return

    if (message.target !== "background")
        return
    
    console.log("message, sender")
    console.log(message, sender, reply)
    reply("from background");
});

// var importObject = { imports: { imported_func: arg => console.log(arg) } };
// WebAssembly.instantiateStreaming(fetch('simple.wasm'), importObject)
// .then(obj => obj.instance.exports.exported_func());//output 42 in console

import init from '/kaspa-ng.js';
(async () => {
    let kaspa_ng = await init('/kaspa-ng_bg.wasm');
console.log(init);
console.log(kaspa_ng);
    // const wasm = await kaspa.default('./kaspa-wallet/kaspa-wallet_bg.wasm');
    await kaspa_ng.kaspa_ng_background();
})();
