
  function log(...args){
    console.log("%c📘[kng-content-script]:", "color:green", ...args);
  }
  
  const EVENT_KEY = (Math.random()*1000000000000).toString().substring(0, 10);
  const EVENT_NAME = "kaspa-wallet-message-"+EVENT_KEY;
  const EVENT_REPLY = "kaspa-wallet-message-reply-"+EVENT_KEY;
  
  log("EVENT_KEY", EVENT_KEY);

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
    port.postMessage({type: "WebAPI", data:event.detail})
  }, false);

  let port = chrome.runtime.connect({name:"CONTENT"});
  port.onMessage.addListener(function(msg) {
    log("msg", msg);
    replyToPage(msg)
  })

  port.postMessage({type: "WebAPI", data:{action:"InjectPageScript", data:[chrome.runtime.id, EVENT_KEY]}});
  