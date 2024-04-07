// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

const queryExports = require('./sql/query_exports.js');
const {contextBridge} = require('electron');

const getprocesses = () => {
  return queryExports.getprocesses();
}


// make this function available to the renderer
// aka making it avaliable to the clien-side code
// aka exposing it to the "Main World"

contextBridge.exposeInMainWorld('api', {
  getprocesses: getprocesses
});
