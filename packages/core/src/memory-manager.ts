type HandleState = 'active' | 'freed'

export class MemoryManager {
  private handles = new Map<bigint, HandleState>()

  register(handle: bigint): void { this.handles.set(handle, 'active') }

  isActive(handle: bigint): boolean { return this.handles.get(handle) === 'active' }

  free(handle: bigint): void {
    const existing = this.handles.get(handle)
    if (existing) this.handles.set(handle, 'freed')
  }

  clear(): void { this.handles.clear() }

  validate(handle: bigint): void {
    const state = this.handles.get(handle)
    if (state === undefined) throw new Error(`Handle ${handle} not found`)
    if (state === 'freed') throw new Error(`Handle ${handle} already freed`)
  }

  stats(): { active: number; freed: number; total: number } {
    let active = 0, freed = 0
    for (const [, s] of this.handles) {
      if (s === 'active') active++
      else freed++
    }
    return { active, freed, total: this.handles.size }
  }
}
