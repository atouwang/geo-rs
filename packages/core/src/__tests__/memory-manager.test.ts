import { describe, it, expect, beforeEach } from 'vitest'
import { MemoryManager } from '../memory-manager'

describe('MemoryManager', () => {
  let mm: MemoryManager

  beforeEach(() => { mm = new MemoryManager() })

  it('registers and validates handles', () => {
    mm.register(1n)
    expect(() => mm.validate(1n)).not.toThrow()
  })

  it('throws on unknown handle', () => {
    expect(() => mm.validate(999n)).toThrow('not found')
  })

  it('throws on freed handle', () => {
    mm.register(1n)
    mm.free(1n)
    expect(() => mm.validate(1n)).toThrow('already freed')
  })

  it('isActive works correctly', () => {
    mm.register(1n)
    expect(mm.isActive(1n)).toBe(true)
    mm.free(1n)
    expect(mm.isActive(1n)).toBe(false)
  })

  it('tracks stats', () => {
    mm.register(1n)
    mm.register(2n)
    mm.free(1n)
    expect(mm.stats()).toEqual({ active: 1, freed: 1, total: 2 })
  })

  it('clears all handles', () => {
    mm.register(1n)
    mm.register(2n)
    mm.clear()
    expect(mm.stats().total).toBe(0)
  })

  it('free() does not throw on unknown handles', () => {
    expect(() => mm.free(999n)).not.toThrow()
  })
})
