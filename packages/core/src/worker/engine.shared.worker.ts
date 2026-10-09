import { attachEngine } from './runtime'

// Share the worker and compiled module, while keeping each client's arena isolated.
;(self as unknown as { onconnect: (event: MessageEvent) => void }).onconnect = (event) => {
  const port = event.ports[0]
  attachEngine(port)
  port.start()
}
