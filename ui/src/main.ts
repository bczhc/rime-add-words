import $ from 'jquery'
import {dialog, tauri} from "@tauri-apps/api";

window.addEventListener("DOMContentLoaded", onLoaded);

function onLoaded() {
    console.log("aa");
    let loadButton = $("#load-button");
    let pickFileButton = $("#pick-file-button");
    let pathSpan = $("#path-span");
    let addButton = $("#add-button");
    let wordInput = $("#word-input");
    let codeInput = $("#code-input");

    let load = false;
    let dictPath: string | null = null;

    pickFileButton.on("click", () => {
        tauri.invoke<string | null>("pick_file").then(file => {
            if (file != null) {
                dictPath = file;
                pathSpan.text(dictPath);
            }
        })
    });

    loadButton.on("click", () => {
        let filePath = pathSpan.text();
        console.log(filePath);
        loadButton.prop("disabled", true);
        tauri.invoke("load_file", {path: filePath})
            .then(() => load = true)
            .catch(() => {
                loadButton.prop("disabled", false);
                dialog.message("Failed to open file").then();
            });
    });

    wordInput.on("input", () => {
        if (!load) {
            return;
        }
        tauri.invoke<string | null>("compose_code", {word: wordInput.val()}).then(result => {
            codeInput.val(result || "");
        })
    });

    addButton.on("click", () => {
        if (!load) {
            return;
        }
        console.assert(dictPath != null);
        tauri.invoke("add_word", {
            word: wordInput.val(),
            code: codeInput.val()
        }).then(() => {
            tauri.invoke("write_to_file", {path: dictPath}).then(() => {
                dialog.message("添加成功").then();
            })
        }).catch(e => {
            dialog.message(`Error: ${e}`).then();
        })
    });
}
