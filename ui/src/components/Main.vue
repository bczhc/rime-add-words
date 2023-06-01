<script setup lang="ts">
import $ from 'jquery'
import Sortable from 'sortablejs';
import {tauri} from '../tauri';
import {useMessage} from 'naive-ui';

const message = useMessage();

window.addEventListener("DOMContentLoaded", onLoaded);

const CIRCLED_NUMBERS = [...'①②③④⑤⑥⑦⑧⑨⓪'] as string[]

let codeInput: JQuery | null = null

async function onLoaded() {
    let wubiContentDiv = $('#wubi-content-div');
    let loadButton = $("#load-button");
    let dictPickFileButton = $("#dict-pick-file");
    let charMapPickFileButton = $("#char-map-pick-file");
    let addButton = $("#add-button");
    codeInput = $("#code-input");
    let addWordsCodeInputEl = $('#add-words-code-input');
    let addWordsWordInputEl = $('#add-words-word-input');
    let dictPathSpan = $("#dict-path-span");
    let charMapPathSpan = $("#char-map-path-span");
    let shortcutCB = $("#shortcut-cb");
    let allInputs = $('input, textarea');

    let ifInputsFocused = () => {
        let anyFocused = false;
        allInputs.each((_i, obj) => {
            if ($(obj).is(':focus')) {
                anyFocused = true;
            }
        });
        return anyFocused;
    };

    interface WordEntry {
        word: string,
        code: string | null,
    }

    let batchAddWordsBox = {
        wordList: [] as WordEntry[],
        wordListIndex: -1,
        textarea: $("#batch-add-words-ta"),
        loadButton: $("#batch-add-words-load-btn"),
        nextButton: $("#batch-add-words-next-btn"),
        previousButton: $("#batch-add-words-previous-btn"),
    };

    let appArgs = await tauri.getAppArgs();
    dictPathSpan.text(appArgs.dictPath || '');
    charMapPathSpan.text(appArgs.charMapPath || '');

    let setWubiDivEnabled = (enabled: boolean) => {
        wubiContentDiv.find('*').prop('disabled', !enabled);
    };
    setWubiDivEnabled(false);

    let wordsList = new WordsList();

    let writeToFile = async () => {
        await tauri.writeToFile(dictPathSpan.text());
    }

    wordsList.onUpdate = async wordList => {
        let code = codeInput!.val() as string;
        await tauri.updateWords(code, wordList)
        await writeToFile()
    }

    codeInput.on("input", async () => {
        await wordsList.updateWordsList()
    });

    dictPickFileButton.on("click", async () => {
        let file = await tauri.pickFile()
        if (file != null) {
            dictPathSpan.text(file);
        }
    });

    charMapPickFileButton.on("click", async () => {
        let file = await tauri.pickFile()
        if (file != null) {
            charMapPathSpan.text(file);
        }
    });

    loadButton.on("click", async () => {
        loadButton.prop("disabled", true);
        try {
            await tauri.loadFile(dictPathSpan.text(), charMapPathSpan.text());
            setWubiDivEnabled(true);
        } catch (e) {
            loadButton.prop("disabled", false);
            message.error(`Failed to open file: ${e}`);
        }
    });

    let updateComposedCode = async () => {
        let result = await tauri.composeCode(addWordsWordInputEl.val() as string);
        addWordsCodeInputEl.val(result || '')
        codeInput!.val(addWordsCodeInputEl.val() as string);
        await wordsList.updateWordsList()
    };

    addWordsWordInputEl.on("input", async () => {
        await updateComposedCode();
    });

    addButton.on("click", async () => {
        try {
            await tauri.addWord(addWordsWordInputEl.val() as string, addWordsCodeInputEl.val() as string)
            await writeToFile()
            message.success('添加成功', {
                duration: 3000
            });
            await wordsList.updateWordsList()
        } catch (e) {
            message.error(`Error: ${e}`);
        }
    });

    batchAddWordsBox.loadButton.on('click', async () => {
        batchAddWordsBox.wordList = (batchAddWordsBox.textarea.val() as string).split('\n')
            .filter(x => x != '')
            .map(x => {
                if (x.match(/\s/)) {
                    let split = x.split(/\s+/);
                    return {
                        word: split[0],
                        code: split[1],
                    }
                } else {
                    return {
                        word: x,
                        code: null
                    }
                }
            });
        batchAddWordsBox.wordListIndex = -1;
        message.info(`成功载入${batchAddWordsBox.wordList.length}条`)
    });
    let batchAddWordsUpdate = async (entry: WordEntry) => {
        addWordsWordInputEl.val(entry.word);
        if (entry.code != null) {
            addWordsCodeInputEl.val(entry.code);
            codeInput!.val(entry.code);
            await wordsList.updateWordsList();
        } else {
            await updateComposedCode();
        }
    };
    batchAddWordsBox.nextButton.on('click', async () => {
        ++batchAddWordsBox.wordListIndex;
        let entry: WordEntry | undefined = batchAddWordsBox.wordList[batchAddWordsBox.wordListIndex];
        if (entry == undefined) {
            message.info('结束');
            --batchAddWordsBox.wordListIndex;
            return
        }
        await batchAddWordsUpdate(entry);
    });
    batchAddWordsBox.previousButton.on('click', async () => {
        --batchAddWordsBox.wordListIndex;
        if (batchAddWordsBox.wordListIndex == -1) {
            message.info('到头了');
            ++batchAddWordsBox.wordListIndex;
            return
        }
        let entry = batchAddWordsBox.wordList[batchAddWordsBox.wordListIndex];
        await batchAddWordsUpdate(entry);
    });

    document.addEventListener('keydown', e => {
        if (!shortcutCB.is(':checked') || ifInputsFocused()) {
            return;
        }
        switch (e.code) {
            case 'KeyA':
                // add word
                addButton.trigger('click');
                break;
            case 'KeyS':
                // previous
                batchAddWordsBox.previousButton.trigger('click');
                break;
            case 'KeyF':
                // next
                batchAddWordsBox.nextButton.trigger('click');
                break;
            default:
        }
    });

    addWordsCodeInputEl.on('input', async () => {
        codeInput!.val(addWordsCodeInputEl.val() as string);
        await wordsList.updateWordsList();
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
</script>

<template>
  Rime词库：<span id="dict-path-span"></span>
    <button id="dict-pick-file">选择文件</button>
    <br>
  单字映射表：<span id="char-map-path-span"></span>
    <button id="char-map-pick-file">选择文件</button>
    <br>
    <button id="load-button">加载</button>
    <br>
    <div id="wubi-content-div">
        <label>
            <input type="text" placeholder="编码" id="code-input">
        </label><br>
        <!--suppress HtmlUnknownTag -->
        <li hidden="hidden" id="li-template">
            <span id="word-span"></span>
            <button id="delete-btn">删除</button>
            <button id="change-position-btn">调</button>
        </li>
        <ul id="words-list">
        </ul>

        <div id="add-words-div" class="bordered-box">
            <span>加词</span><br>
            <label for="add-words-word-input"></label><input type="text" id="add-words-word-input" placeholder="词条">
            <br>
            <label for="add-words-code-input"></label><input type="text" id="add-words-code-input" placeholder="编码">
            <br>
            <button id="add-button">添加</button>
        </div>
        <div class="bordered-box">
            <span>批量加词</span><br>

            <label for="batch-add-words-ta"></label>
            <textarea id="batch-add-words-ta" placeholder="词条列表（格式：word[<space> code]）" cols="50"
                      rows="5"></textarea>
            <br>
            <button id="batch-add-words-load-btn">载入</button>
            <button id="batch-add-words-previous-btn">上一个</button>
            <button id="batch-add-words-next-btn">下一个</button>
            快捷键<input type="checkbox" id="shortcut-cb">
        </div>
    </div>
</template>

<style scoped>
#words-list > li {
    font-size: large;
    user-select: none;
    font-family: sans-serif;
    border: solid 1px gray;
    margin-bottom: -1px;
    padding-left: 4px;
}

#words-list {
    list-style: none;
    padding: 0;
    margin: 0;
}

/* noinspection CssUnusedSymbol */
.blue-background {
    background-color: aqua;
}

.bordered-box {
    border: solid 1px black;
    padding: 5px;
    margin-top: 3px;
    margin-bottom: 3px;
}
</style>
