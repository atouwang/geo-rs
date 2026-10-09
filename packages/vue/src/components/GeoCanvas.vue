<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { renderGeometry, type CanvasGeometry } from './render-geometry'

const props = defineProps<{ width?: number; height?: number; geometry?: CanvasGeometry | null }>()
const canvasRef = ref<HTMLCanvasElement>()
function render() {
  const canvas = canvasRef.value
  const ctx = canvas?.getContext('2d')
  if (canvas && ctx) renderGeometry(ctx, canvas.width, canvas.height, props.geometry ?? null)
}
watch(() => [props.geometry, props.width, props.height], render, { deep: true, flush: 'post' })
onMounted(render)
</script>

<template>
  <canvas ref="canvasRef" :width="width ?? 800" :height="height ?? 600" class="geo-canvas" />
</template>

<style scoped>
.geo-canvas { display: block; border: 1px solid #30363d; border-radius: 4px; background: #0d1117; }
</style>

