<script setup lang="ts">
</script>

<template>
  <div class="app">
    <header class="app-header">
      <h1><router-link to="/">geo-rs</router-link></h1>
      <span class="badge">Rust + WASM</span>
      <nav>
        <router-link to="/">Home</router-link>
        <router-link to="/benchmarks">Benchmarks</router-link>
        <router-link to="/playground">Playground</router-link>
        <a href="https://github.com/geo-rs/geo-rs" target="_blank">GitHub</a>
      </nav>
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
  --accent-orange: #d2991d;
  --accent-red: #f85149;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg);
  color: var(--text);
  min-height: 100vh;
  line-height: 1.6;
}

a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }

.app { max-width: 1100px; margin: 0 auto; padding: 24px; }

.app-header {
  display: flex; align-items: center; gap: 16px;
  padding-bottom: 20px; border-bottom: 1px solid var(--border);
  margin-bottom: 32px; flex-wrap: wrap;
}

.app-header h1 { font-size: 24px; font-weight: 700; }
.app-header h1 a { color: var(--text); }

.badge {
  font-size: 11px; color: var(--accent);
  background: rgba(88, 166, 255, 0.1);
  padding: 2px 10px; border-radius: 12px; font-weight: 500;
}

nav { margin-left: auto; display: flex; gap: 20px; }
nav a { font-size: 14px; color: var(--text-dim); }
nav a:hover, nav a.router-link-active { color: var(--text); text-decoration: none; }

section { margin-bottom: 48px; }
h2 { font-size: 22px; margin-bottom: 12px; }
h3 { font-size: 16px; margin-bottom: 12px; color: var(--text-dim); }
p { color: var(--text-dim); font-size: 15px; margin-bottom: 8px; }
</style>
