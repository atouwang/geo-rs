<script setup lang="ts">
const benchData = [
  { op: 'area', time: '46 ns' },
  { op: 'R-tree search (10K pts)', time: '33 ns' },
  { op: 'GeoJSON parse', time: '473 ns' },
  { op: 'centroid', time: '239 ns' },
  { op: 'buffer', time: '943 ns' },
  { op: 'contains', time: '2.0 us' },
  { op: 'union (two polygons)', time: '5.8 us' },
  { op: 'simplify (1000 pts)', time: '54 us' },
]
const features = [
  { title: 'Buffer', desc: 'Polygon inflation/deflation via straight skeleton' },
  { title: 'Set Operations', desc: 'Union, intersection, difference, XOR' },
  { title: 'Spatial Predicates', desc: 'All 8 DE-9IM relations' },
  { title: 'Spatial Indexing', desc: 'R*-Tree and KD-Tree' },
  { title: 'Voronoi & Grids', desc: 'Voronoi, hex, square, triangle grids' },
  { title: 'Isolines', desc: 'Marching Squares contour extraction' },
  { title: 'Coordinate Transforms', desc: 'WGS84, Web Mercator, Cartesian' },
  { title: 'Web Worker + WASM', desc: 'Non-blocking dedicated thread' },
]
</script>

<template>
  <section class="hero">
    <h2>Browser-Native Geospatial Analysis</h2>
    <p>Rust-powered spatial operations compiled to WebAssembly. Drop-in faster alternative to Turf.js.</p>
  </section>
  <section>
    <h3>Performance Highlights</h3>
    <div class="bench-grid">
      <div v-for="b in benchData" :key="b.op" class="bench-card">
        <div class="bench-time">{{ b.time }}</div>
        <div class="bench-op">{{ b.op }}</div>
      </div>
    </div>
    <p style="margin-top:12px"><router-link to="/benchmarks">View full benchmarks &rarr;</router-link></p>
  </section>
  <section>
    <h3>Supported Operations</h3>
    <div class="feature-grid">
      <div v-for="f in features" :key="f.title" class="feature-card">
        <h4>{{ f.title }}</h4>
        <p>{{ f.desc }}</p>
      </div>
    </div>
  </section>
  <section>
    <h3>Quick Start</h3>
    <pre class="code-block"><code>npm install @geo-rs/core

import { buffer, intersect } from '@geo-rs/core'

const zone = await buffer(cityCenter, { radius: 500, units: 'meters' })
const overlap = await intersect(zone, buildings)</code></pre>
  </section>
</template>

<style scoped>
.hero h2 { font-size: 28px; margin-bottom: 12px; }
.bench-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 10px; }
.bench-card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 16px; text-align: center; }
.bench-time { font-size: 24px; font-weight: 700; color: var(--accent-green); font-family: monospace; }
.bench-op { font-size: 12px; color: var(--text-dim); margin-top: 4px; }
.feature-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 10px; }
.feature-card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 16px; }
.feature-card h4 { font-size: 14px; margin-bottom: 6px; }
.feature-card p { font-size: 13px; color: var(--text-dim); }
.code-block { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 20px; overflow-x: auto; }
.code-block code { font-family: monospace; font-size: 13px; line-height: 1.7; }
</style>
