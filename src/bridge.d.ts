import NativeLib from './NativeLib';

declare global {
  interface Window {
    nativeLib: NativeLib
  }
}
