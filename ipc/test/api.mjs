class runtime{
    constructor(chrome){
        this.chrome = chrome;
        this.onMessage = {
            addListener: (callback)=>{
                this.chrome.onMessageListeners.push(callback);
            }
        }
    }

    sendMessage(msg, callback){
        for (let chrome of chromes){
            if (chrome == this.chrome)
                continue;

            for (let listener of chrome.onMessageListeners){
                listener(msg, {id: this.chrome.id}, callback)
            }
        }
    }
}
const chromes = [];
export class Chrome{
    constructor(){
        this.id = "xxxxxx";
        this.onMessageListeners = [];
        this.runtime = new runtime(this);
        chromes.push(this);
    }
    
}