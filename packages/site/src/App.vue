<script setup lang="ts">
</script>

<template>
  <div class="app">
    <header class="app-header">
      <h1>geo-rs</h1>
      <span class="badge">WASM Spatial Analysis</span>
    </header>
    <main>
      <router-view />
    </main>
  </div>
</template>

<style>
:root {
  --bg: #0d1117;
  --surface: #161b22;
  --border: #30363d;
  --text: #e6edf3;
  --text-dim: #8b949e;
  --accent: #58a6ff;
  --accent-green: #3fb950;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg);
  color: var(--text);
  min-height: 100vh;
}

.app { max-width: 1200px; margin: 0 auto; padding: 24px; }

.app-header {
  display: flex; align-items: baseline; gap: 12px;
  padding-bottom: 24px; border-bottom: 1px solid var(--border);
  margin-bottom: 32px;
}

.app-header h1 { font-size: 28px; font-weight: 700; }

.badge {
  font-size: 12px; color: var(--accent);
  background: rgba(88, 166, 255, 0.1);
  padding: 2px 8px; border-radius: 12px;
}
</style>
