// fetch('/kaspa-egui-57c7a8dd13e092be_bg.wasm')
//   //.then(response => response.arrayBuffer())
//   .then(response => {
//     // const blob = new Blob([bytes], { type: 'application/wasm' });
//     // const url = URL.createObjectURL(blob);

//     // Now, you can create a Wasm instance using the URL
//     return WebAssembly.instantiateStreaming(response, WebAssembly.);
//   })
//   .then(result => {
//     const instance = result.instance;
//     console.log("instance", instance)
//     // Use the Wasm module
//   })
//   .catch(error => {
//     console.error('Failed to load Wasm module:', error);
//   });
const OFFSCREEN_DOCUMENT_PATH = "/offscreen.html";

chrome.runtime.onInstalled.addListener(async () => {
    await createOffscreenDoc();
});

async function createOffscreenDoc(){
    // Create an offscreen document if one doesn't exist yet
    if ((await hasDocument()))
        return

    await chrome.offscreen.createDocument({
        url: OFFSCREEN_DOCUMENT_PATH,
        reasons: [chrome.offscreen.Reason.DOM_PARSER],
        justification: 'Parse DOM'
    });
}

async function closeOffscreenDocument() {
    if (!(await hasDocument())) {
        return;
    }
    await chrome.offscreen.closeDocument();
}

async function hasDocument() {
    // Check all windows controlled by the service worker if one of them is the offscreen document
    const matchedClients = await clients.matchAll();
    for (const client of matchedClients) {
        if (client.url.endsWith(OFFSCREEN_DOCUMENT_PATH)) {
        return true;
        }
    }
    return false;
}

chrome.runtime.onMessage.addListener((message, sender, reply)=>{
    console.log("message, sender")
    console.log(message, sender, reply)
    if (sender.id != chrome.runtime.id)
        return

    if (message.target !== "background")
        return
    reply("from background");
});
