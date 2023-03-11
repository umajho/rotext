<script lang="ts">
import { NCard, NInput } from "naive-ui";
import RendererVue from "./renderer-vue.vue";
</script>

<template>
  <n-input class="input" v-model:value="markup" type="textarea"></n-input>

  <div style="height: 4vh"></div>

  <n-card class="container">
    <renderer-vue
      :markup="markup"
      @error="(e) => emit('error', e as Error)"
      @update-character-count="(c) => emit('updateCharacterCount', c)"
      @update-parsing-time="(tMs) => emit('updateParsingTime', tMs)"
    ></renderer-vue>
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
import { onMounted, ref, watch } from "vue";

const emit = defineEmits<{
  (event: "updateCharacterCount", count: number): void;
  (event: "updateParsingTime", timeMs: number): void;
  (event: "error", error: Error | null): void;
}>();

let markup = ref(localStorage.getItem("markup") ?? "");
watch(markup, (updated) => {
  localStorage.setItem("markup", updated);
});

onMounted(() => {
  emit("error", null);
});
</script>
