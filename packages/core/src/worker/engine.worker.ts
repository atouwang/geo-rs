import { attachEngine, type EnginePort } from './runtime'

attachEngine(self as unknown as EnginePort)
