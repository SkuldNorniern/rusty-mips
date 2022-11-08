interface MyLib {
  hello: () => string
}

declare global {
  interface Window {
    nativeLib: MyLib
  }
}

export {};
