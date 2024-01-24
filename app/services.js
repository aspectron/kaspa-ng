(() => {

    const blob = new Blob ([`
    var idMap = {};
    self.addEventListener('message', function(event) {
        let { name, id, time }  = event.data;
        switch (name) {
            case 'setInterval':
                idMap[id] = setInterval(function () {
                    postMessage({id});
                }, time);
                break;
            case 'clearInterval':
                if (idMap.hasOwnProperty(id)) {
                    clearInterval(idMap[id]);
                    delete idMap[id];
                }
                break;
            case 'setTimeout':
                idMap[id] = setTimeout(function () {
                    postMessage({id});
                    if (idMap.hasOwnProperty(id)) {
                        delete idMap[id];
                    }
                }, time);
                break;
            case 'clearTimeout':
                if (idMap.hasOwnProperty(id)) {
                    clearTimeout(idMap[id]);
                    delete idMap[id];
                }
                break;
        }
    });
    `]);
    
    const workerScript = window.URL.createObjectURL(blob);

    const maxId = 0x7FFFFFFF;
    let lastId = 0;
    let callbacks = {};

    function getId () {
        do {
            if (lastId == maxId) {
                lastId = 0;
            } else {
                lastId ++;
            }
        } while (callbacks.hasOwnProperty (lastId));
        return lastId;
    }

    try {
        worker = new Worker (workerScript);
        window.setInterval = function (callback, time /* , parameters */) {
            const id = getId();
            callbacks[id] = {
                callback: () => { callback() },
                parameters: Array.prototype.slice.call(arguments, 2)
            };
            worker.postMessage ({
                name: 'setInterval',
                id,
                time,
            });
            return id;
        };
        window.clearInterval = function (id) {
            if (callbacks.hasOwnProperty(id)) {
                delete callbacks[id];
                worker.postMessage ({
                    name: 'clearInterval',
                    id,
                });
            }
        };
        window.setTimeout = function (callback, time /* , parameters */) {
            const id = getId();
            callbacks[id] = {
                callback: callback,
                parameters: Array.prototype.slice.call(arguments, 2),
                isTimeout: true
            };
            worker.postMessage ({
                name: 'setTimeout',
                id,
                time,
            });
            return id;
        };

        window.clearTimeout = function (id) {
            if (callbacks.hasOwnProperty(id)) {
                delete callbacks[id];
                worker.postMessage ({
                    name: 'clearTimeout',
                    id,
                });
            }
        };

        worker.addEventListener('message',function (event) {
            let data = event.data,
                id = data.id,
                request;
            if (callbacks.hasOwnProperty(id)) {
                request = callbacks[id];
                let { callback, parameters, isTimeout } = request;
                if (isTimeout) {
                    delete callbacks[id];
                }
                callback.apply (window, parameters);
            } else {
                console.error("services: unknown timer id:", id);
            }
        });

    } catch (error) {
        console.log ('timers.js - initialization failed');
        console.error (error);
    }

})();