export const API_VERSION = 1;

export interface NativeLib {
  init: (refresh: (state: object) => void) => number
  finalize: () => void
  reset: () => void

  assemble: (code: string, endian: string) => string | null
  editRegister: (idx: number, value: number) => void
  readMemory: (pageIdx: number, dst: Uint8Array) => Uint8Array | null
  step: () => void
  run: (useJit: boolean) => void
  stop: () => void
  getNativeEndian: () => 'big' | 'little'
  convertToPipeline: () => void
}

interface IDisassembly {
  [k: string]: [number, string]
}

interface IPipelineDetailEntry {
  name: string
  value: string
}

interface IPipelineDetail {
  [k: string]: IPipelineDetailEntry
}

interface IModuleStateRaw {
  regs: number[]
  pc: number
  running: boolean
  disasm: IDisassembly
  disasmList: number[]
  cleanAfterReset: boolean
  canUseJit: boolean
  canUsePipeline: boolean
  pipelineDetail: IPipelineDetail
  pipelineDetailList: string[]
}
export type IModuleState = Readonly<IModuleStateRaw>;
