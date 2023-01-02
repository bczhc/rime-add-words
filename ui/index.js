import jquery from 'jquery'
import {dialog, tauri} from "@tauri-apps/api";

window.$ = jquery
window.tauriDialog = dialog;
window.tauriInvoke = tauri.invoke;
