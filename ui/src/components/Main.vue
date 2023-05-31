<script>
import '../main'
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
