window.sendToPlugin = function (msg) {
    // window.ipc.postMessage(JSON.stringify(msg));
    window.ipc.postMessage(msg);
  }
  
  window.onPluginMessage = function() {};
  
  window.onPluginMessageInternal = function(msg) {
    // console.log(msg);
    // const json = JSON.parse(msg);
    // window.onPluginMessage && window.onPluginMessage(json);
    window.onPluginMessage && window.onPluginMessage(msg);
  }