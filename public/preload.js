const nativeLib = require('./index.node');
const { contextBridge } = require('electron');
contextBridge.exposeInMainWorld('nativeLib', nativeLib);
