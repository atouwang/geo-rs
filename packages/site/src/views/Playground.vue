<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { encode, decode } from '@msgpack/msgpack'

const input = ref('{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}')
const operation = ref('area')
const output = ref('')
const status = ref('')
const wasmReady = ref(false)
const wasmError = ref('')
const stats = ref('')

let engine: any = null

const examples: Record<string, string> = {
  Square: '{"type":"Polygon","coordinates":[[[0,0],[10,0],[10,10],[0,10],[0,0]]]}',
  Triangle: '{"type":"Polygon","coordinates":[[[0,0],[10,0],[5,10],[0,0]]]}',
  Beijing: '{"type":"Point","coordinates":[116.397,39.908]}',
}

onMounted(async () => {
  try {
    const wasm = await import('geo-wasm')
    await wasm.init()
    engine = new wasm.Engine()
    wasmReady.value = true
    stats.value = engine.stats()
  } catch (err) {
    wasmError.value = err instanceof Error ? err.message : String(err)
  }
})

async function run() {
  if (!engine) return
  status.value = 'running'
  output.value = ''
  try {
    const geom = JSON.parse(input.value)
    const data = encode(geom)
    const h = engine.load(data)

    if (operation.value === 'area') {
      const area = engine.execute_measure(0x01, h)
      output.value = JSON.stringify({ area }, null, 2)
    } else if (operation.value === 'length') {
      const len = engine.execute_measure(0x02, h)
      output.value = JSON.stringify({ length: len }, null, 2)
    } else if (operation.value === 'buffer') {
      const h2 = engine.execute_unary(0x10, h, 0.5)
      const bytes = engine.read(h2)
      const result = decode(bytes)
      output.value = JSON.stringify(result, null, 2)
      engine.free(h2)
    } else if (operation.value === 'centroid') {
      const h2 = engine.execute_unary(0x03, h, 0)
      const bytes = engine.read(h2)
      const result = decode(bytes)
      output.value = JSON.stringify(result, null, 2)
      engine.free(h2)
    } else if (operation.value === 'simplify') {
      const h2 = engine.execute_unary(0x11, h, 0.5)
      const bytes = engine.read(h2)
      const result = decode(bytes)
      output.value = JSON.stringify(result, null, 2)
      engine.free(h2)
    }
    engine.free(h)
    stats.value = engine.stats()
    status.value = 'done'
  } catch (e) {
    output.value = JSON.stringify({ error: String(e) }, null, 2)
    status.value = 'error'
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
