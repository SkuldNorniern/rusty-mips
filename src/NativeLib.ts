export const API_VERSION = 1;

interface NativeLib {
  init: () => number
  finalize: () => void
}

export default NativeLib;
