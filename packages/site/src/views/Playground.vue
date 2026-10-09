<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { GeoEngine, type GeoJSON } from '@geo-rs/core'

const input = ref('{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}')
const operation = ref('area')
const output = ref('')
const status = ref('')
const wasmReady = ref(false)
const wasmError = ref('')
const stats = ref('')
let engine: GeoEngine | null = null
let disposed = false

const examples: Record<string, string> = {
  Square: '{"type":"Polygon","coordinates":[[[0,0],[10,0],[10,10],[0,10],[0,0]]]}',
  Triangle: '{"type":"Polygon","coordinates":[[[0,0],[10,0],[5,10],[0,0]]]}',
  Beijing: '{"type":"Point","coordinates":[116.397,39.908]}',
}

onMounted(async () => {
  try {
    const initialized = await GeoEngine.init()
    if (disposed) { initialized.destroy(); return }
    engine = initialized
    stats.value = JSON.stringify(await initialized.stats())
    wasmReady.value = true
  } catch (error) {
    if (!disposed) wasmError.value = error instanceof Error ? error.message : String(error)
  }
})

onUnmounted(() => {
  disposed = true
  engine?.destroy()
  engine = null
})

async function run() {
  const current = engine
  if (!current || status.value === 'running') return
  status.value = 'running'
  output.value = ''
  const handles: bigint[] = []
  try {
    const h = await current.load(JSON.parse(input.value) as GeoJSON)
    handles.push(h)
    let result: unknown
    if (operation.value === 'area') {
      result = { area: await current.area(h) }
    } else if (operation.value === 'length') {
      result = { length: await current.length(h) }
    } else {
      const h2 = operation.value === 'buffer' ? await current.buffer(h, 0.5)
        : operation.value === 'centroid' ? await current.centroid(h)
          : await current.simplify(h, 0.5)
      handles.push(h2)
      result = await current.read(h2)
    }
    if (!disposed) {
      output.value = JSON.stringify(result, null, 2)
      status.value = 'done'
    }
  } catch (error) {
    if (!disposed) {
      output.value = JSON.stringify({ error: String(error) }, null, 2)
      status.value = 'error'
    }
  } finally {
    current.free(...handles)
    if (!disposed) {
      try { stats.value = JSON.stringify(await current.stats()) } catch { /* Worker may have crashed. */ }
    }
  }
}
</script>

<template>
  <section>
    <h2>Playground</h2>

    <div v-if="wasmError" class="banner error">
      WASM init failed: {{ wasmError }}
    </div>
    <div v-else-if="!wasmReady" class="banner">
      Loading WASM engine...
    </div>

    <p v-if="wasmReady">GeoJSON input, pick operation, see result. Engine: {{ stats }}</p>
    <p v-else>GeoJSON input, pick operation, see result.</p>

    <div class="playground">
      <div class="panel">
        <h3>Input</h3>
        <div class="examples">
          <button v-for="(val, key) in examples" :key="key" @click="input = val" class="btn-sm">{{ key }}</button>
        </div>
        <textarea v-model="input" rows="8" spellcheck="false" class="code-input" />
        <div class="controls">
          <select v-model="operation">
            <option value="area">area()</option>
            <option value="length">length()</option>
            <option value="buffer">buffer()</option>
            <option value="centroid">centroid()</option>
            <option value="simplify">simplify()</option>
          </select>
          <button @click="run" class="btn-run" :disabled="!wasmReady || status === 'running'">
            {{ status === 'running' ? 'Running...' : 'Run' }}
          </button>
        </div>
      </div>
      <div class="panel">
        <h3>Output</h3>
        <pre class="code-output" :class="status">{{ output || '// Result here' }}</pre>
      </div>
    </div>
  </section>
</template>

<style scoped>
.banner {
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  font-size: 14px;
}
.banner.error {
  border-color: var(--accent-red);
  color: var(--accent-red);
}
.playground { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
@media (max-width: 768px) { .playground { grid-template-columns: 1fr; } }
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 16px; }
.panel h3 { margin-bottom: 10px; font-size: 13px; color: var(--text-dim); }
.examples { display: flex; gap: 6px; margin-bottom: 10px; flex-wrap: wrap; }
.btn-sm { padding: 3px 10px; font-size: 12px; background: var(--surface); border: 1px solid var(--border); border-radius: 4px; color: var(--text-dim); cursor: pointer; }
.btn-sm:hover { color: var(--text); }
.code-input { width: 100%; background: var(--bg); border: 1px solid var(--border); border-radius: 4px; padding: 12px; color: var(--text); font-family: monospace; font-size: 13px; resize: vertical; }
.controls { display: flex; gap: 10px; margin-top: 12px; align-items: center; }
select { padding: 6px 12px; background: var(--bg); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.btn-run { padding: 6px 20px; background: var(--accent); border: none; border-radius: 4px; color: #fff; font-size: 13px; font-weight: 600; cursor: pointer; }
.btn-run:hover { opacity: 0.9; }
.btn-run:disabled { opacity: 0.5; cursor: not-allowed; }
.code-output { background: var(--bg); border: 1px solid var(--border); border-radius: 4px; padding: 12px; font-family: monospace; font-size: 13px; color: var(--text-dim); min-height: 200px; white-space: pre-wrap; word-break: break-all; }
.code-output.done { color: var(--accent-green); }
.code-output.error { color: var(--accent-red); }
</style>
