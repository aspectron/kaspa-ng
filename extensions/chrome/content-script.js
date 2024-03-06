
  function log(...args){
    console.log("%c📘[kng-content-script]:", "color:green", ...args);
  }
  
  const EVENT_KEY = (Math.random()*1000000000000).toString().substring(0, 10);
  const EVENT_NAME = "kaspa-wallet-message-"+EVENT_KEY;
  const EVENT_REPLY = "kaspa-wallet-message-reply-"+EVENT_KEY;
  
  log("EVENT_KEY", EVENT_KEY);
  const ICON = chrome.runtime.getURL("icons/icon-128.png");
  chrome.runtime.sendMessage({op: "inject-page-script", args: [chrome.runtime.id, ICON, EVENT_KEY]})
  
  function replyToPage(detail){
    window.dispatchEvent(new CustomEvent(EVENT_REPLY, {
      detail
    }));
  }
  
  window.addEventListener(EVENT_NAME, async (event) => {
    log("message event", event.source === window, event.source, event);
  
    if (event.target !== window) {
      return;
    }
  
    log("event.detail", event.detail);
  
    //forward msg to extension
    let response = await chrome.runtime.sendMessage(event.detail);
    log("sendMessage: response", response)
    //window.open(chrome.runtime.getURL("popup.html"), self, "width=400,left=100,height=800,frame=0")
    if (!response)
      return true
    //reply to page
    replyToPage(response)
  }, false);
  
  //listen extension and forward message to page
  chrome.runtime.onMessage.addListener(async (msg, sender, sendResponse) => {
    log("extension message", msg, sender)
    if (sender.id !== chrome.runtime.id)
      return;
  
      replyToPage(msg)
  });
  
  