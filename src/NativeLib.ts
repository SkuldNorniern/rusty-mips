export const API_VERSION = 1;

export interface NativeLib {
  init: (refresh: (state: object) => void) => number
  finalize: () => void
  reset: () => void

  assemble: (code: string, endian: string) => string | null
  editRegister: (idx: number, value: number) => void
  readMemory: (pageIdx: number, dst: Uint8Array) => Uint8Array
  step: () => void
  run: () => void
  stop: () => void
  getNativeEndian: () => 'big' | 'little'
}

interface IDisassembly {
  [k: string]: [number, string]
}

interface IModuleStateRaw {
  regs: number[]
  pc: number
  running: boolean
  disasm: IDisassembly
  disasmList: number[]
  cleanAfterReset: boolean
}
export type IModuleState = Readonly<IModuleStateRaw>;
