async function apiBuilder(uuid, eventKey) {

    const EVENT_NAME = "kaspa-wallet-message-" + eventKey;
    const EVENT_REPLY = "kaspa-wallet-message-reply-" + eventKey;

    const EVENT_WALLET_DISCOVERY = "kaspa-wallet-discovery";
    const EVENT_WALLET_ANNOUNCE = "kaspa-wallet-announce";

    function log(...args) {
        console.log("📘[kaspa-api]:", ...args);
    }

    log("event key", eventKey);
    let events = new Map();

    function postMessage(action, data, rid) {
        rid = rid === false ? undefined : rid || action;
        let result;
        if (rid) {
            result = new Promise((resolve, reject) => {
                events.set(rid, { resolve, reject });
            });
        } else {
            result = Promise.resolved();
        }

        window.dispatchEvent(new CustomEvent(EVENT_NAME, {
            detail: { action, data, rid }
        }));
        return result;
    }

    window.addEventListener(EVENT_REPLY, (msg) => {
        let { data, rid } = msg.detail;

        log("reply:", msg.detail);
        if (rid && events.has(rid)) {
            if (data.error){
                events.get(rid).reject(data)
            }else{
                events.get(rid).resolve(data)
            }
            
            events.delete(rid);
        }
    })

    class KaspaApi {

        connect() {
            // Communicate a message back to the extension
            return postMessage("Connect");
        }

        signTransaction(data) {
            // Communicate a message back to the extension
            return postMessage("SignTransaction", data);
        }
        
        testRequestResponse(data) {
            return postMessage("TestRequestResponse", data);
        }
    }

    const kaspa = new KaspaApi();

    function annouceWallet() {
        let detail = Object.freeze({
            uuid,
            name: "Kaspa NG",
            icon: "data:image/png;base64,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",
            api: kaspa
        })

        window.dispatchEvent(new CustomEvent(EVENT_WALLET_ANNOUNCE, {
            detail
        }));
    }

    window.addEventListener(EVENT_WALLET_DISCOVERY, () => {
        annouceWallet()
    });

    annouceWallet();


    //temporary, 
    if (!window.discover_kaspa_wallet) {
        let wallets = [];
        window.addEventListener(EVENT_WALLET_ANNOUNCE, (event) => {
            wallets.push(event.detail)
        });
        window.discover_kaspa_wallet = () => {
            wallets = [];
            window.dispatchEvent(new CustomEvent(EVENT_WALLET_DISCOVERY));
            return wallets
        }
    }

    // //temporary, just for testing via console
    // window.kaspa = kaspa;
    // // // testing ...
    // kaspa.signTransaction({tx_info:"xyz"}).then(result=>{
    //     console.log("kaspa.signTransaction", result)
    // })
}

export { apiBuilder };