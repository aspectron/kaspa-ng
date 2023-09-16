// We disable caching during development so that we always view the latest version.
// if ('serviceWorker' in navigator && window.location.hash !== "#dev") {
//     window.addEventListener('load', function () {
//         navigator.serviceWorker.register('./pwa.js');
//     });
// }

// import init from '/kaspa-egui-57c7a8dd13e092be.js';
// init('/kaspa-egui-57c7a8dd13e092be_bg.wasm');

// fetch('/kaspa-egui-57c7a8dd13e092be_bg.wasm')
//   .then(response => response.arrayBuffer())
//   .then(bytes => {
//     const blob = new Blob([bytes], { type: 'application/wasm' });
//     const url = URL.createObjectURL(blob);

//     // Now, you can create a Wasm instance using the URL
//     return WebAssembly.instantiateStreaming(fetch(url));
//   })
//   .then(result => {
//     const instance = result.instance;
//     console.log("instance", instance)
//     // Use the Wasm module
//   })
//   .catch(error => {
//     console.error('Failed to load Wasm module:', error);
//   });

document.querySelector("#btn").addEventListener("click", ()=>{
    chrome.runtime.sendMessage({
        target: "offscreen",
        data: "message from popup"
    }, (msg)=>{
        if (msg){
            alert("msg:"+msg);
        }
    })
})

chrome.runtime.onMessage.addListener((message, sender, reply)=>{
    console.log("popup:", message)
    let {data} = message;
    if (data?.counter){
        document.querySelector("#counter").textContent = `counter: ${data.counter}`;
    }
})