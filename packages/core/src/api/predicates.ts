import type { GeoJSON } from '../types'
import { getSharedEngine, releaseSharedEngine } from './shared'

export async function contains(a: GeoJSON, b: GeoJSON): Promise<boolean> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const ha = await engine.load(a)
    handles.push(ha)
    const hb = await engine.load(b)
    handles.push(hb)
    const result = await engine.contains(ha, hb)
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function intersects(a: GeoJSON, b: GeoJSON): Promise<boolean> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const ha = await engine.load(a)
    handles.push(ha)
    const hb = await engine.load(b)
    handles.push(hb)
    const result = await engine.intersects(ha, hb)
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}

export async function crosses(a: GeoJSON, b: GeoJSON): Promise<boolean> {
  const engine = await getSharedEngine()
  const handles: bigint[] = []
  try {
    const ha = await engine.load(a)
    handles.push(ha)
    const hb = await engine.load(b)
    handles.push(hb)
    const result = await engine.crosses(ha, hb)
    return result
  } finally {
    engine.free(...handles)
    releaseSharedEngine()
  }
}
