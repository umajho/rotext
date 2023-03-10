<script lang="ts">
import { NAlert, NCard, NInput, NTag } from "naive-ui";
</script>

<template>
  <n-tag :bordered="false" type="info"> 字数：{{ characterCount }} </n-tag>
  <span style="display: inline-block; width: 4vw"></span>
  <n-tag :bordered="false" type="info"> 解析时间：{{ parsingTimeMs }}ms </n-tag>
  <span style="display: inline-block; width: 4vw"></span>
  <n-tag :bordered="false" type="info">
    渲染时间：{{ renderingTimeMs }}ms
  </n-tag>

  <div style="height: 2vh"></div>

  <n-input
    id="input"
    v-model:value="markup"
    type="textarea"
    @input="renderMarkup()"
  ></n-input>

  <div style="height: 4vh"></div>

  <template v-if="error">
    <n-alert type="error" :title="error.message">
      {{ error.stack }}
    </n-alert>
    <div style="height: 4vh"></div>
  </template>

  <main>
    <n-card id="container">
      <div ref="outputEl"></div>
    </n-card>
  </main>
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
import { onMounted, type Ref, ref } from "vue";

import {
  classModule,
  // eventListenersModule,
  init,
  // datasetModule,
  // propsModule,
  styleModule,
  type VNode,
} from "snabbdom";

import { parse } from "rotext-renderer-snabbdom";

let markup = ref(localStorage.getItem("markup") ?? "");
const outputEl: Ref<HTMLElement | null> = ref(null);

let lastNode: VNode | HTMLElement;
let patch: ReturnType<typeof init> | null = null;

let error: Ref<Error | null> = ref(null);

let characterCount = ref(0);
let parsingTimeMs = ref(0);
let renderingTimeMs = ref(0);

onMounted(() => {
  patch = init(
    [
      classModule,
      // propsModule,
      // datasetModule,
      styleModule,
      // eventListenersModule,
    ],
    undefined,
    { experimental: { fragments: true } }
  );
  lastNode = outputEl.value!;

  renderMarkup();
});

function renderMarkup() {
  error.value = null;
  localStorage.setItem("markup", markup.value);
  characterCount.value = [...new Intl.Segmenter().segment(markup.value)].length;
  try {
    const parsingStart = performance.now();
    const currentNode = parse(markup.value, { breaks: true });
    parsingTimeMs.value = performance.now() - parsingStart;

    if (!lastNode) {
      patch!(outputEl.value!, currentNode);
    } else {
      patch!(lastNode, currentNode);
    }
    lastNode = currentNode;

    const renderingStart = performance.now();
    requestAnimationFrame(() => {
      renderingTimeMs.value = performance.now() - renderingStart;
    });
  } catch (e) {
    if (!(e instanceof Error)) {
      e = new Error(`${e}`);
    }
    error.value = e as Error;
  }
}
</script>
