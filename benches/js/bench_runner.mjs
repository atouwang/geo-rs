// Simple benchmark comparing geo-rs operations to Turf.js
// Run:  node bench_runner.mjs

import { Buffer } from 'node:buffer';

// --- Test data generation ---
function createSimplePolygon() {
    return {
        type: "Polygon",
        coordinates: [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]
    };
}

function createMultiPolygon(count) {
    const polygons = [];
    for (let i = 0; i < count; i++) {
        const ox = (i % 10) * 2;
        const oy = Math.floor(i / 10) * 2;
        polygons.push([[
            [ox, oy], [ox + 1, oy], [ox + 1, oy + 1], [ox, oy + 1], [ox, oy]
        ]]);
    }
    return { type: "MultiPolygon", coordinates: polygons };
}

function createPoints(count) {
    const coords = [];
    for (let i = 0; i < count; i++) {
        coords.push([Math.random() * 10, Math.random() * 10]);
    }
    return { type: "MultiPoint", coordinates: coords };
}

// --- Benchmark runner ---
class BenchRunner {
    constructor() {
        this.results = [];
    }

    bench(name, fn, iterations = 1000) {
        // Warmup
        for (let i = 0; i < 10; i++) fn();

        const start = performance.now();
        for (let i = 0; i < iterations; i++) fn();
        const elapsed = performance.now() - start;
        const opsPerSec = (iterations / elapsed) * 1000;

        this.results.push({ name, opsPerSec, elapsedMs: elapsed, iterations });
        console.log(`  ${name}: ${opsPerSec.toFixed(1)} ops/s (${elapsed.toFixed(1)}ms for ${iterations} iters)`);
    }

    summary() {
        console.log('\n=== Benchmark Summary ===');
        const sorted = [...this.results].sort((a, b) => b.opsPerSec - a.opsPerSec);
        for (const r of sorted) {
            console.log(`  ${r.name.padEnd(40)} ${r.opsPerSec.toFixed(1)} ops/s`);
        }
    }
}

// --- Turf.js benchmarks (baseline) ---
async function benchTurf() {
    const turf = await import('@turf/turf');
    console.log('\n--- Turf.js Benchmarks ---');
    const runner = new BenchRunner();

    const poly = createSimplePolygon();
    const mp = createMultiPolygon(100);
    const points = createPoints(1000);

    runner.bench('turf_area_simple', () => {
        turf.area(poly);
    }, 5000);

    runner.bench('turf_centroid_simple', () => {
        turf.centroid(poly);
    }, 5000);

    runner.bench('turf_buffer_simple', () => {
        turf.buffer(poly, 0.5, { units: 'kilometers' });
    }, 100);

    runner.bench('turf_simplify', () => {
        const line = turf.lineString(points.coordinates.map(p => [p[0], p[1]]));
        turf.simplify(line, { tolerance: 0.01 });
    }, 100);

    runner.bench('turf_contains_point', () => {
        turf.booleanContains(poly, turf.point([0.5, 0.5]));
    }, 5000);

    runner.bench('turf_union', () => {
        const b = turf.polygon([[[0.5, 0.5], [1.5, 0.5], [1.5, 1.5], [0.5, 1.5], [0.5, 0.5]]]);
        turf.union(poly, b);
    }, 100);

    return runner;
}

// --- Main ---
console.log('geo-rs Benchmark Suite');
console.log('======================');

try {
    const runner = await benchTurf();
    runner.summary();
    console.log('\nGeo-rs WASM benchmarks pending — run after wasm-pack build');
    console.log('(JS fallback comparison data ready)');
} catch (err) {
    console.error('Benchmark error:', err.message);
    process.exit(1);
}
