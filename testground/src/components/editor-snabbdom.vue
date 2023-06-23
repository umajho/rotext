<script lang="ts">
import { NCard, NInput } from "naive-ui";
</script>

<template>
  <n-input class="input" v-model:value="markup" type="textarea" @input="renderMarkup()"></n-input>

  <div style="height: 4vh"></div>

  <n-card class="container">
    <div class="previewer">
      <div ref="outputEl"></div>
    </div>
  </n-card>
</template>

<style>
.input {
  width: 80vw;
  height: calc(100vh / 3);
  overflow: scroll;
  text-align: left;
}

.container {
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

import { parse } from "@rotext-lite/renderer-snabbdom";

const emit = defineEmits<{
  (event: "updateCharacterCount", count: number): void;
  (event: "updateParsingTime", timeMs: number): void;
  (event: "error", error: Error | null): void;
}>();

let markup = ref(localStorage.getItem("markup") ?? "");
const outputEl: Ref<HTMLElement | null> = ref(null);

let lastNode: VNode | HTMLElement;
let patch: ReturnType<typeof init> | null = null;

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
  emit("error", null);
  localStorage.setItem("markup", markup.value);
  const charCount = [...new Intl.Segmenter().segment(markup.value)].length;
  emit("updateCharacterCount", charCount);
  try {
    const parsingStart = performance.now();
    const currentNode = parse(markup.value, { breaks: true });
    emit("updateParsingTime", performance.now() - parsingStart);

    if (!lastNode) {
      patch!(outputEl.value!, currentNode);
    } else {
      patch!(lastNode, currentNode);
    }
    lastNode = currentNode;
  } catch (e) {
    if (!(e instanceof Error)) {
      e = new Error(`${e}`);
    }
    emit("error", e as Error);
  }
}
</script>
