export const API_VERSION = 1;

export interface NativeLib {
  init: (refresh: (state: object) => void) => number
  finalize: () => void
  reset: () => void

  assemble: (code: string) => string | null
}

interface IModuleStateRaw {
  regs: number[]
}
export type IModuleState = Readonly<IModuleStateRaw>;
