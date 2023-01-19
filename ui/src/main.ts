import $ from 'jquery'
import {dialog} from "@tauri-apps/api";
import Sortable from 'sortablejs';
import {tauri} from './tauri';

window.addEventListener("DOMContentLoaded", onLoaded);

function onLoaded() {
    let wubiContentDiv = $('#wubi-content-div');
    let loadButton = $("#load-button");
    let pickFileButton = $("#pick-file-button");
    let pathSpan = $("#path-span");
    let addButton = $("#add-button");
    let codeInput = $("#code-input");
    let addWordsCodeInputEl = $('#add-words-code-input');
    let addWordsWordInputEl = $('#add-words-word-input');

    let setWubiDivEnabled = (enabled: boolean) => {
        wubiContentDiv.find('*').prop('disabled', !enabled);
    };
    setWubiDivEnabled(false);

    let queriedWords: string[] = []

    let wordsList = new WordsList(queriedWords);
    let dictPath: string | null = null;

    let writeToFile = async () => {
        await tauri.writeToFile(dictPath!);
    }

    wordsList.onMoveEnd = data => {
        let code = codeInput.val() as string;
        tauri.updateWords(code, data).then(() => {
            writeToFile().then();
        });
    }
    wordsList.onDelete = position => {
        queriedWords.splice(position, 1);
        wordsList.refreshList();
        let code = codeInput.val() as string;
        tauri.updateWords(code, queriedWords).then(() => {
            writeToFile().then();
        })
    };

    let updateWordsList = () => {
        let code = codeInput.val() as string;
        let promise = tauri.queryWords(code);
        promise.then(words => {
            queriedWords.length = 0;
            queriedWords.push(...words);
            wordsList.refreshList();
        })
    };
    codeInput.on("input", updateWordsList);

    pickFileButton.on("click", () => {
        tauri.pickFile().then(file => {
            if (file != null) {
                dictPath = file;
                pathSpan.text(dictPath);
            }
        });
    });

    loadButton.on("click", () => {
        loadButton.prop("disabled", true);
        tauri.loadFile(dictPath!)
            .then(() => {
                setWubiDivEnabled(true);
            })
            .catch((e) => {
                loadButton.prop("disabled", false);
                dialog.message(`Failed to open file: ${e}`).then();
            });
    });

    addWordsWordInputEl.on("input", () => {
        tauri.composeCode(addWordsWordInputEl.val() as string).then(result => {
            addWordsCodeInputEl.val(result || '');
            codeInput.val(addWordsCodeInputEl.val() as string);
            updateWordsList();
        })
    });

    addButton.on("click", () => {
        tauri.addWord(addWordsWordInputEl.val() as string, addWordsCodeInputEl.val() as string).then(() => {
            writeToFile().then(() => {
                dialog.message("添加成功").then();
                updateWordsList()
            });
        }).catch(e => {
            dialog.message(`Error: ${e}`).then();
        })
    });
}

class WordsList {
    private listEl = $('#words-list');
    private readonly data: string[]

    public onMoveEnd: ((data: string[]) => void) | null = null

    public onDelete: ((position: number) => void) | null = null

    constructor(data: string[]) {
        this.data = data;
        Sortable.create(this.listEl[0], {
            animation: 150,
            ghostClass: 'blue-background',
            direction: _ => 'vertical',
            onEnd: event => {
                if (this.data == null) {
                    return
                }
                let oldIndex = event.oldIndex!;
                let newIndex = event.newIndex!;
                let removed = this.data.splice(oldIndex, 1)[0];
                this.data.splice(newIndex, 0, removed);
                this.onMoveEnd?.(this.data);
            }
        });
    }

    private createLiElement(text: string) {
        let element = $('#li-template')
            .clone()
            .prop('hidden', false);
        element.find('#word-span').text(text)
        element.find('#delete-btn').on('click', () => {
            let index = -1;
            let children = this.listEl.children();
            for (let i = 0; i < children.length; i++) {
                if (children[i] == element[0]) {
                    index = i;
                }
            }
            this.onDelete?.(index);
        });
        return element;
    }

    public refreshList() {
        this.listEl.empty();
        for (let word of this.data) {
            let element = this.createLiElement(word);
            this.listEl.append(element);
        }
    }
}
