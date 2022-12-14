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

export const ToastProvider = ({ children }: IProps): JSX.Element => {
  const [state, setState] = React.useState<Array<[string, string]>>([]);

  const onShowToast = (title: string, value: string): void => {
    setState(prev => [...prev, [title, value]]);
    setTimeout(() => setState(prev => [...prev.slice(1)]), 5000);
  };

  return (
    <>
      <Fullscreen>
        <ToastContainer position="top-end">
          {state.map(x => <Toast key={`${x[0]}=${x[1]}`}>
            <Toast.Header><Title>{x[0]}</Title></Toast.Header>
            <Toast.Body>{x[1]}</Toast.Body>
          </Toast>)}
        </ToastContainer>
      </Fullscreen>
      <ToastContext.Provider value={{ showToast: onShowToast }}>
        {children}
      </ToastContext.Provider>
    </>
  );
};
