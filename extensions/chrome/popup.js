// We disable caching during development so that we always view the latest version.
// if ('serviceWorker' in navigator && window.location.hash !== "#dev") {
//     window.addEventListener('load', function () {
//         navigator.serviceWorker.register('./pwa.js');
//     });
// }

// import init from '/kaspa-egui-57c7a8dd13e092be.js';
// init('/kaspa-egui-57c7a8dd13e092be_bg.wasm');

// -----------

// document.querySelector("#btn").addEventListener("click", ()=>{
//     chrome.runtime.sendMessage({
//         target: "offscreen",
//         data: "message from popup"
//     }, (msg)=>{
//         if (msg){
//             alert("msg:"+msg);
//         }
//     })
// })

// chrome.runtime.onMessage.addListener((message, sender, reply)=>{
//     console.log("popup:", message)
//     let {data} = message;
//     if (data?.counter){
//         document.querySelector("#counter").textContent = `counter: ${data.counter}`;
//     }
// })

// import init from '/kaspa-ng.js';
// let kaspa_ng = init('/kaspa-egui_bg.wasm');
import init from '/kaspa-ng.js';
(async () => {
    let kaspa_ng = await init('/kaspa-ng_bg.wasm');

    // const wasm = await kaspa.default('./kaspa-wallet/kaspa-wallet_bg.wasm');
    await kaspa_ng.kaspa_ng_main();
})();


// wasm_bindgen('/kaspa-ng_bg.wasm');