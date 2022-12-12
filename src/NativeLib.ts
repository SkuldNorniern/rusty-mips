export const API_VERSION = 1;

export interface NativeLib {
  init: (refresh: (state: object) => void) => number
  finalize: () => void
  reset: () => void

  assemble: (code: string) => string | null
  editRegister: (idx: number, value: number) => void
  readMemory: (pageIdx: number, dst: Uint8Array) => boolean
}

interface IModuleStateRaw {
  regs: number[]
  pc: number
}
export type IModuleState = Readonly<IModuleStateRaw>;
