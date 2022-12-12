import React from 'react';
import { API_VERSION, NativeLib, IModuleState } from '../NativeLib';

interface INativeModuleRaw {
  lib: NativeLib
  initialized: boolean
  loadError: boolean
  state: IModuleState
}
export type INativeModule = Readonly<INativeModuleRaw>;

function defaultModule (loadError: boolean): INativeModule {
  return {
    lib: window.nativeLib,
    initialized: false,
    loadError,
    state: undefined as unknown as IModuleState
  };
}

export const NativeLibContext = React.createContext<INativeModule>(defaultModule(true));

interface Props {
  children: JSX.Element
}

export const NativeLibProvider = ({ children }: Props): JSX.Element => {
  const [module, setModule] = React.useState<INativeModule>(defaultModule(false));

  const onRefresh = (updates: any): void => {
    setModule((prev) => {
      if (prev.initialized) {
        return { ...prev, state: { ...prev.state, ...updates } };
      } else {
        return { ...prev, initialized: true, state: updates };
      }
    });
  };

  React.useEffect(() => {
    const libVersion = window.nativeLib.init(onRefresh);
    if (libVersion !== API_VERSION) {
      console.error(`Expected api version = ${API_VERSION}, but linked library has api version = ${libVersion}`);
      window.nativeLib.finalize();
      setModule({ ...module, loadError: true });
    } else {
      console.log(`Initialized native module, api version = ${libVersion}`);
      // We don't update here. Wait for module to update for us.
    }

    return function cleanup () {
      console.log('Cleaning up native module');
      window.nativeLib.finalize();
    };
  }, []);

  return (
    <NativeLibContext.Provider value={module}>
      {children}
    </NativeLibContext.Provider>
  );
};
