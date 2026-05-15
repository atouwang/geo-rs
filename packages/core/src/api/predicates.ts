import { GeoEngine } from '../engine'
import type { GeoJSON } from '../types'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function contains(a: GeoJSON, b: GeoJSON): Promise<boolean> {
  const engine = await getSharedEngine()
  try {
    const ha = await engine.load(a)
    const hb = await engine.load(b)
    const result = await engine.contains(ha, hb)
    engine.free(ha, hb)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function intersects(a: GeoJSON, b: GeoJSON): Promise<boolean> {
  const engine = await getSharedEngine()
  try {
    const ha = await engine.load(a)
    const hb = await engine.load(b)
    const result = await engine.intersects(ha, hb)
    engine.free(ha, hb)
    return result
  } finally {
    releaseSharedEngine()
  }
}

export async function crosses(a: GeoJSON, b: GeoJSON): Promise<boolean> {
  const engine = await getSharedEngine()
  try {
    const ha = await engine.load(a)
    const hb = await engine.load(b)
    const result = await engine.crosses(ha, hb)
    engine.free(ha, hb)
    return result
  } finally {
    releaseSharedEngine()
  }
}
