export class MemoryManager {
  private handles = new Map<bigint, number>()
  private freed = new Set<bigint>()
  private static readonly historyLimit = 1024

  register(handle: bigint): void {
    this.freed.delete(handle)
    this.handles.set(handle, (this.handles.get(handle) ?? 0) + 1)
  }

  isActive(handle: bigint): boolean { return (this.handles.get(handle) ?? 0) > 0 }

  free(handle: bigint): void {
    const existing = this.handles.get(handle)
    if (!existing) return
    if (existing > 1) { this.handles.set(handle, existing - 1); return }
    this.handles.delete(handle)
    this.freed.add(handle)
    if (this.freed.size > MemoryManager.historyLimit) this.freed.delete(this.freed.values().next().value!)
  }

  clear(): void { this.handles.clear(); this.freed.clear() }

  validate(handle: bigint): void {
    const state = this.handles.get(handle)
    if (state === undefined) {
      if (this.freed.has(handle)) throw new Error(`Handle ${handle} already freed`)
      throw new Error(`Handle ${handle} not found`)
    }
  }

  stats(): { active: number; freed: number; total: number } {
    return { active: this.handles.size, freed: this.freed.size, total: this.handles.size + this.freed.size }
  }
}
