export class MemoryManager {
  private handles = new Map<bigint, number>()

  register(handle: bigint): void { this.handles.set(handle, (this.handles.get(handle) ?? 0) + 1) }

  isActive(handle: bigint): boolean { return (this.handles.get(handle) ?? 0) > 0 }

  free(handle: bigint): void {
    const existing = this.handles.get(handle)
    if (existing) this.handles.set(handle, existing - 1)
  }

  clear(): void { this.handles.clear() }

  validate(handle: bigint): void {
    const state = this.handles.get(handle)
    if (state === undefined) throw new Error(`Handle ${handle} not found`)
    if (state === 0) throw new Error(`Handle ${handle} already freed`)
  }

  stats(): { active: number; freed: number; total: number } {
    let active = 0, freed = 0
    for (const [, s] of this.handles) {
      if (s > 0) active++
      else freed++
    }
    return { active, freed, total: this.handles.size }
  }
}
