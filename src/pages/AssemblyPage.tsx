import React from 'react';
import Button from 'react-bootstrap/Button';
import styled from '@emotion/styled';
import { NativeLibContext } from '../context/NativeLibContext';

const Root = styled.div`
  display: flex;
  flex-direction: column;
  padding: 1rem;
  height: 100%;
`;

const CodeArea = styled.textarea`
  flex: 1;
  margin-bottom: 1rem;
`;

const StatusArea = styled.div`
  display: flex;
  flex-direction: row;
`;

const StatusText = styled.div`
  word-break: break-word;
`;

const ButtonArea = styled.div`
  flex-shrink: 0;
  margin-left: auto;
  margin-right: 0;
`;

const defaultValue = `
.text
.globl main

main:
  jr $ra
`;

const AssemblyPage = (): JSX.Element => {
  const native = React.useContext(NativeLibContext);
  const codeRef = React.useRef<HTMLTextAreaElement>(null);
  const [lastStatus, setLastStatus] = React.useState('');

  const onClickErase = (): void => {
    if (codeRef.current != null) {
      codeRef.current.value = '';
      setLastStatus('');
    }
  };

  const onAssemble = (): void => {
    if (codeRef.current == null || !native.initialized) { return; }
    const answer = native.lib.assemble(codeRef.current.value);
    if (answer != null) {
      setLastStatus(answer);
    } else {
      setLastStatus('assemble success');
    }
  };

  const onReset = (): void => {
    if (native.initialized) {
      native.lib.reset();
      setLastStatus('');
    }
  };

  return (
    <Root>
      <CodeArea ref={codeRef} className="code" defaultValue={defaultValue.trim()} />
      <StatusArea>
        <StatusText>
          {lastStatus}
        </StatusText>
        <ButtonArea>
          <Button variant="primary" onClick={onAssemble}>어셈블!</Button>{' '}
          <Button variant="secondary" onClick={onClickErase}>지우기</Button>{' '}
          <Button variant="danger" onClick={onReset}>시뮬레이터 리셋</Button>
        </ButtonArea>
      </StatusArea>
    </Root>
  );
};

export default AssemblyPage;
