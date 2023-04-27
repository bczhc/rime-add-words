import $ from 'jquery'
import {dialog} from "@tauri-apps/api";
import Sortable from 'sortablejs';
import {tauri} from './tauri';

window.addEventListener("DOMContentLoaded", onLoaded);

const CIRCLED_NUMBERS = [...'①②③④⑤⑥⑦⑧⑨⓪'] as string[]

let codeInput: JQuery | null = null

function onLoaded() {
    let wubiContentDiv = $('#wubi-content-div');
    let loadButton = $("#load-button");
    let pickFileButton = $("#pick-file-button");
    let pathSpan = $("#path-span");
    let addButton = $("#add-button");
    codeInput = $("#code-input");
    let addWordsCodeInputEl = $('#add-words-code-input');
    let addWordsWordInputEl = $('#add-words-word-input');

    let setWubiDivEnabled = (enabled: boolean) => {
        wubiContentDiv.find('*').prop('disabled', !enabled);
    };
    setWubiDivEnabled(false);

    let wordsList = new WordsList();
    let dictPath: string | null = null;

    let writeToFile = async () => {
        await tauri.writeToFile(dictPath!);
    }

    wordsList.onUpdate = async wordList => {
        let code = codeInput!.val() as string;
        await tauri.updateWords(code, wordList)
        await writeToFile()
    }

    codeInput.on("input", async () => {
        await wordsList.updateWordsList()
    });

    pickFileButton.on("click", async () => {
        let file = await tauri.pickFile()
        if (file != null) {
            dictPath = file;
            pathSpan.text(dictPath);
        }
    });

    loadButton.on("click", async () => {
        loadButton.prop("disabled", true);
        try {
            await tauri.loadFile(dictPath!)
            setWubiDivEnabled(true);
        } catch (e) {
            loadButton.prop("disabled", false);
            dialog.message(`Failed to open file: ${e}`).then();
        }
    });

    addWordsWordInputEl.on("input", async () => {
        let result = await tauri.composeCode(addWordsWordInputEl.val() as string);
        addWordsCodeInputEl.val(result || '')
        codeInput!.val(addWordsCodeInputEl.val() as string);
        await wordsList.updateWordsList()
    });

    addButton.on("click", async () => {
        try {
            await tauri.addWord(addWordsWordInputEl.val() as string, addWordsCodeInputEl.val() as string)
            await writeToFile()
            await dialog.message('添加成功')
            await wordsList.updateWordsList()
        } catch (e) {
            dialog.message(`Error: ${e}`).then();
        }
    });
}

class WordsList {
    private listEl = $('#words-list');
    public readonly data: string[]

    public onUpdate: ((wordList: string[]) => void) | null = null

    constructor() {
        this.data = [];
        Sortable.create(this.listEl[0], {
            animation: 150,
            ghostClass: 'blue-background',
            direction: _ => 'vertical',
            onEnd: async event => {
                if (this.data == null) {
                    return
                }
                let oldIndex = event.oldIndex!;
                let newIndex = event.newIndex!;
                let removed = this.data.splice(oldIndex, 1)[0];
                this.data.splice(newIndex, 0, removed);
                // update entry ordinal numbers
                this.onUpdate?.(this.data)
                await this.updateWordsList()
            }
        });
    }

    private createLiElement(text: string) {
        let element = $('#li-template')
            .clone()
            .prop('hidden', false);
        element.find('#word-span').text(text)
        element.find('#delete-btn').on('click', async () => {
            let index = -1;
            let children = this.listEl.children();
            for (let i = 0; i < children.length; i++) {
                if (children[i] == element[0]) {
                    index = i;
                }
            }
            this.data.splice(index, 1);
            this.onUpdate?.(this.data)
            await this.updateWordsList()
        });
        element.find('#change-position-btn').on('click', async () => {
            let index = -1;
            let children = this.listEl.children();
            for (let i = 0; i < children.length; i++) {
                if (children[i] == element[0]) {
                    index = i;
                }
            }

            let code = codeInput!.val() as string;
            let wordList = await tauri.queryWords(code) as (string | undefined)[]
            let input = prompt('新的位置')
            if (input == null) return
            let position = parseInt(input);
            if (isNaN(position)) return
            if (position == 0) position = 10
            let oldWord = wordList[index]
            wordList[index] = undefined
            wordList[position - 1] = oldWord
            for (let i = 0; i < wordList.length; i++) {
                if (wordList[i] == undefined) {
                    wordList[i] = CIRCLED_NUMBERS[i]
                }
            }
            this.data.length = 0
            this.data.push(...wordList as string[])
            this.onUpdate?.(this.data)
            await this.updateWordsList()
        })
        return element;
    }

    public refreshList() {
        this.listEl.empty();
        for (let i = 0; i < this.data.length; i++) {
            let word = this.data[i]
            let element = this.createLiElement(`${i + 1} ${word}`)
            this.listEl.append(element)
        }
    }

    public async updateWordsList() {
        let code = codeInput!.val() as string;
        let words = await tauri.queryWords(code);
        this.data.length = 0
        this.data.push(...words)
        this.refreshList()
    }
}
