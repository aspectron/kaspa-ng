chrome.runtime.onMessage.addListener((message, sender, reply)=>{
    if (sender.id != chrome.runtime.id)
        return

    if (message.target !== "background")
        return
    
    console.log("message, sender")
    console.log(message, sender, reply)
    reply("from background");
});
