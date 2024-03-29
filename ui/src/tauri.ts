import {invoke} from "@tauri-apps/api";

export module tauri {
    export async function queryWords(code: string) {
        return await invoke<string[]>('query_words', {
            code: code,
        });
    }

    export async function pickFile() {
        return await invoke<string | null>('pick_file');
    }

    export async function loadFile(dictPath: string, charMapPath: string) {
        return await invoke<void>('load_file', {
            dictPath: dictPath,
            charMapPath: charMapPath
        });
    }

    export async function composeCode(word: string) {
        return await invoke<string | null>('compose_code', {
            word: word,
        });
    }

    export async function addWord(word: string, code: string) {
        return await invoke<void>('add_word', {
            word: word,
            code: code,
        })
    }

    export async function writeToFile(path: string) {
        return await invoke<void>('write_to_file', {
            path: path,
        });
    }

    export async function updateWords(code: string, words: string[]) {
        return await invoke<void>('update_words', {
            code: code,
            words: words,
        })
    }

    interface AppArgs {
        dictPath: string | null,
        charMapPath: string | null,
    }

    export async function getAppArgs() {
        return await invoke<AppArgs>('get_app_args');
    }
}
