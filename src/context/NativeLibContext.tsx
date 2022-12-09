import React from 'react';
import NativeLib, { API_VERSION } from '../NativeLib';

/** null = library not yet loaded, undefined = incompatible version, linking error, etc. */
export const NativeLibContext = React.createContext<NativeLib | null | undefined>(null);

interface Props {
  children: JSX.Element
}

export const NativeLibProvider = ({ children }: Props): JSX.Element => {
  const [lib, setLib] = React.useState<NativeLib | undefined | null>(null);

  React.useEffect(() => {
    const libVersion = window.nativeLib.init();
    if (libVersion !== API_VERSION) {
      console.error(`Expected api version = ${API_VERSION}, but linked library has api version = ${libVersion}`);
      setLib(undefined);
    } else {
      console.log(`Initialized native module, api version = ${libVersion}`);
      setLib(window.nativeLib);
    }

    return function cleanup () {
      console.log('Cleaning up native module');
      window.nativeLib.finalize();
    };
  }, []);

  return (
    <NativeLibContext.Provider value={lib}>
      {children}
    </NativeLibContext.Provider>
  );
};
