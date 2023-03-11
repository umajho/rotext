<script lang="ts">
import { NAlert, NRadio, NSpace, NTag } from "naive-ui";
import EditorSnabbdom from "./editor-snabbdom.vue";
import EditorVue from "./editor-vue.vue";
</script>

<template>
  <n-space style="background-color: white">
    <n-radio :checked="renderer === 'vue'" value="vue" @change="changeRenderer"
      >Vue</n-radio
    >
    <n-radio
      :checked="renderer === 'snabbdom'"
      value="snabbdom"
      @change="changeRenderer"
      >Snabbdom</n-radio
    >
  </n-space>
  <div style="height: 2vh"></div>

  <n-tag :bordered="false" type="info"> 字数：{{ characterCount }} </n-tag>
  <span style="display: inline-block; width: 4vw"></span>
  <n-tag :bordered="false" type="info"> 解析时间：{{ parsingTimeMs }}ms </n-tag>
  <span style="display: inline-block; width: 4vw"></span>
  <n-tag :bordered="false" type="info"> 渲染时间：？ </n-tag>

  <div style="height: 2vh"></div>

  <template v-if="error">
    <n-alert type="error" :title="error.message">
      {{ error.stack }}
    </n-alert>
    <div style="height: 4vh"></div>
  </template>

  <editor-vue
    v-if="renderer === 'vue'"
    @error="(e) => (error = e)"
    @update-character-count="(c) => (characterCount = c)"
    @update-parsing-time="(tMs) => (parsingTimeMs = tMs)"
  >
  </editor-vue>
  <editor-snabbdom
    v-else-if="renderer === 'snabbdom'"
    @error="(e) => (error = e)"
    @update-character-count="(c) => (characterCount = c)"
    @update-parsing-time="(tMs) => (parsingTimeMs = tMs)"
  >
  </editor-snabbdom>
</template>

<style>
#input {
  width: 80vw;
  height: calc(100vh / 3);
  overflow: scroll;
  text-align: left;
}
#container {
  width: 75vw;
  height: calc(40vh);
  margin: 0 auto;
  overflow: scroll;
  text-align: left;
  border: solid;
}
</style>

<script setup lang="ts">
import { type Ref, ref } from "vue";

let renderer: Ref<"vue" | "snabbdom"> = ref("vue");

let error: Ref<Error | null> = ref(null);

let characterCount = ref(0);
let parsingTimeMs = ref(0);
// let renderingTimeMs = ref(0);

function changeRenderer(e: Event) {
  renderer.value = (e.target as HTMLInputElement).value as unknown as any;
}
</script>
