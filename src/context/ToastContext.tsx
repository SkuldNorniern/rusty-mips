import React from 'react';
import styled from '@emotion/styled';
import ToastContainer from 'react-bootstrap/ToastContainer';
import Toast from 'react-bootstrap/Toast';

interface IToastContext {
  showToast: (title: string, content: string) => void
}

export const ToastContext = React.createContext<Readonly<IToastContext>>({
  showToast: () => console.error('toast used without context')
});

const Fullscreen = styled.div`
  position: absolute;
  left: 5%;
  right: 5%;
  top: 5%;
  bottom: 5%;
  pointer-events: none;
`;

const Title = styled.div`
  margin-right: auto;
`;

interface IProps {
  children: JSX.Element | undefined | null
}

interface IState {
  toasts: Map<number, ToastInfo>
  nextKey: number
}

interface ToastInfo {
  title: string
  value: string
  isShown: boolean
}

export const ToastProvider = ({ children }: IProps): JSX.Element => {
  const [state, setState] = React.useState<Readonly<IState>>({
    toasts: new Map(),
    nextKey: 0
  });

  const onRemoveToast = (toastKey: number): void => {
    setState(prev => {
      const toasts = new Map(prev.toasts);
      toasts.delete(toastKey);
      return {
        ...prev,
        toasts
      };
    });
  };

  const onHideToast = (toastKey: number): void => {
    setState(prev => {
      const toasts = new Map(prev.toasts);
      const info = toasts.get(toastKey);
      if (info != null) {
        info.isShown = false;
        setTimeout(() => onRemoveToast(toastKey), 1000);
      }
      return { ...prev, toasts };
    });
  };

  const onShowToast = (title: string, value: string): void => {
    setState(prev => {
      const currKey = prev.nextKey;
      const toasts = new Map(prev.toasts);
      toasts.set(currKey, { title, value, isShown: true });
      return {
        ...prev,
        toasts,
        nextKey: (currKey + 1) & 0x7fffffff
      };
    });
  };

  const arr = [];
  for (const k of state.toasts.keys()) {
    const toast = state.toasts.get(k);
    if (toast != null) {
      arr.push(
        <Toast key={k} show={toast.isShown} onClose={() => onHideToast(k)} delay={3000} autohide>
          <Toast.Header><Title>{toast.title}</Title></Toast.Header>
          <Toast.Body>{toast.value}</Toast.Body>
        </Toast>
      );
    }
  }

  return (
    <>
      <Fullscreen>
        <ToastContainer position="top-end">
          {arr}
        </ToastContainer>
      </Fullscreen>
      <ToastContext.Provider value={{ showToast: onShowToast }}>
        {children}
      </ToastContext.Provider>
    </>
  );
};
