<script setup lang="ts">
defineProps<{ width?: number; height?: number }>()
</script>

<template>
  <canvas :width="width ?? 800" :height="height ?? 600" class="geo-canvas" />
</template>

<style scoped>
.geo-canvas {
  display: block;
  border: 1px solid #333;
  border-radius: 4px;
}
</style>
