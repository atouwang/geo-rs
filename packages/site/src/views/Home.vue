<script setup lang="ts">
</script>

<template>
  <div class="home">
    <section class="hero">
      <h2>Browser-Native Geospatial Analysis</h2>
      <p>Rust-powered spatial operations compiled to WebAssembly. <strong>Drop-in faster</strong> alternative to Turf.js.</p>
    </section>

    <section class="status">
      <h3>Project Status</h3>
      <div class="status-grid">
        <div class="status-card done">
          <span class="dot" />
          <span>Rust Workspace (7 crates)</span>
        </div>
        <div class="status-card done">
          <span class="dot" />
          <span>geo-core types, measure, convert</span>
        </div>
        <div class="status-card in-progress">
          <span class="dot" />
          <span>WASM Bridge & TS SDK</span>
        </div>
        <div class="status-card pending">
          <span class="dot" />
          <span>Playground & Benchmarks</span>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.hero { margin-bottom: 48px; }
.hero h2 { font-size: 24px; margin-bottom: 8px; }
.hero p { color: var(--text-dim); font-size: 16px; }

.status h3 { margin-bottom: 16px; }

.status-grid {
  display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.status-card {
  display: flex; align-items: center; gap: 8px;
  padding: 12px 16px; background: var(--surface);
  border: 1px solid var(--border); border-radius: 8px;
  font-size: 14px;
}

.dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.done .dot { background: var(--accent-green); }
.in-progress .dot { background: var(--accent); animation: pulse 1.5s infinite; }
.pending .dot { background: var(--text-dim); }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
