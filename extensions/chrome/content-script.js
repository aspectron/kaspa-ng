
function log(...args){
    console.log("%c📘[KNG-api-handler]:", "color:green", ...args);
  }
  
  const eventKey = (Math.random()*10000000000).toString().substring(0, 10);
  const eventName = "kaspa-wallet-message-"+eventKey;
  const eventReplyName = "kaspa-wallet-message-reply-"+eventKey;
  
  log("eventKey", eventKey);
  
  chrome.runtime.sendMessage({op: "inject-page-script", args: [eventKey]})
  
  
  function postToPage(detail){
    window.dispatchEvent(new CustomEvent(eventReplyName, {
      detail
    }));
  }
  
  
  window.addEventListener(eventName, async (event) => {
    log("message event", event.source === window, event);
  
    // We only accept messages from ourselves
    if (event.target !== window) {
      return;
    }
  
    log("event.data ", event.detail);
  
    //forward msg to extension
    chrome.runtime.sendMessage(event.detail, (response)=>{
      log("sendMessage:event.detail", response)
      //window.open(chrome.runtime.getURL("popup.html"), self, "width=400,left=100,height=800,frame=0")
      
      if (!response)
        return
      //reply to page
  
      
      postToPage(response)
    });
  }, false);
  
  //listen extension and forward message to page
  chrome.runtime.onMessage.addListener(async (msg, sender, sendResponse) => {
    //log("extension message", msg, sender)
    if (sender.id !== chrome.runtime.id)
      return;
  
      postToPage(msg)
  });
  
  