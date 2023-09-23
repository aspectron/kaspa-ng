class RunTime{
    constructor(chrome, id){
        this.id = id;
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
                listener(msg, {id: this.id}, callback)
            }
        }
    }
}
const chromes = [];
export class Chrome{
    constructor(id = "xxxxxx"){
        this.onMessageListeners = [];
        this.runtime = new RunTime(this, id);
        chromes.push(this);
    }
    
}