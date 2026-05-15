<script setup lang="ts">
const data = [
  { op: 'area (simple polygon)', rust: '46 ns', turf: '~800 ns', speedup: '17x' },
  { op: 'centroid', rust: '239 ns', turf: '~2 us', speedup: '8x' },
  { op: 'GeoJSON parse', rust: '473 ns', turf: '~3 us', speedup: '6x' },
  { op: 'buffer', rust: '943 ns', turf: '~15 us', speedup: '16x' },
  { op: 'contains (point in poly)', rust: '2.0 us', turf: '~10 us', speedup: '5x' },
  { op: 'union (two squares)', rust: '5.8 us', turf: '~80 us', speedup: '14x' },
  { op: 'R-tree search (10K pts)', rust: '33 ns', turf: '~5 us', speedup: '150x' },
  { op: 'simplify (1000 pts)', rust: '54 us', turf: '~200 us', speedup: '4x' },
]
function color(s: string) {
  const n = parseInt(s)
  if (n >= 50) return 'var(--accent-green)'
  if (n >= 10) return 'var(--accent)'
  return 'var(--accent-orange)'
}
</script>

<template>
  <section>
    <h2>Benchmarks: geo-rs vs Turf.js</h2>
    <p>Rust 1.95 release + LTO. Single-operation latency.</p>
    <div class="table-wrap">
      <table>
        <thead><tr><th>Operation</th><th>geo-rs</th><th>Turf.js</th><th>Speedup</th></tr></thead>
        <tbody>
          <tr v-for="b in data" :key="b.op">
            <td>{{ b.op }}</td><td class="mono">{{ b.rust }}</td>
            <td class="mono dim">{{ b.turf }}</td>
            <td class="mono" :style="{ color: color(b.speedup) }">{{ b.speedup }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.table-wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: 8px; }
table { width: 100%; border-collapse: collapse; font-size: 14px; }
th { text-align: left; padding: 12px 16px; background: var(--surface); border-bottom: 1px solid var(--border); font-weight: 600; font-size: 13px; color: var(--text-dim); }
td { padding: 10px 16px; border-bottom: 1px solid var(--border); }
tr:last-child td { border-bottom: none; }
.mono { font-family: monospace; }
.dim { color: var(--text-dim); }
</style>
