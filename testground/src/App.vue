<script setup lang="ts">
import { darkTheme, NConfigProvider } from "naive-ui";
import TestGround from "./components/test-ground.vue";
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <test-ground></test-ground>
  </n-config-provider>
</template>
