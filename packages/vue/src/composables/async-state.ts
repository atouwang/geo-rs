import { getCurrentScope, onScopeDispose, ref, shallowRef, type Ref, type ShallowRef } from 'vue'

export interface AsyncState<T, Args extends unknown[]> {
  execute: (...args: Args) => Promise<void>
  result: ShallowRef<T | null>
  loading: Ref<boolean>
  error: Ref<Error | null>
}

export function useAsyncState<T, Args extends unknown[]>(operation: (...args: Args) => Promise<T>): AsyncState<T, Args> {
  const loading = ref(false)
  const error = ref<Error | null>(null)
  const result: ShallowRef<T | null> = shallowRef(null)
  let sequence = 0, pending = 0, disposed = false
  if (getCurrentScope()) onScopeDispose(() => { disposed = true; sequence++; loading.value = false })

  async function execute(...args: Args): Promise<void> {
    if (disposed) return
    const current = ++sequence
    pending++
    loading.value = true
    error.value = null
    try {
      const value = await operation(...args)
      if (!disposed && current === sequence) result.value = value
    } catch (reason) {
      if (!disposed && current === sequence) error.value = reason instanceof Error ? reason : new Error(String(reason))
    } finally {
      pending--
      if (!disposed) loading.value = pending > 0
    }
  }
  return { execute, result, loading, error }
}
