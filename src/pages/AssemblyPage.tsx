import React from 'react';
import Button from 'react-bootstrap/Button';
import styled from '@emotion/styled';

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

const ButtonArea = styled.div`
  margin-left: auto;
  margin-right: 0;
`;

const defaultValue = `
.globl main
.text

main:
  jr $ra
`;

const AssemblyPage = (): JSX.Element => {
  const codeRef = React.useRef<HTMLTextAreaElement>(null);

  const onClickErase = (): void => {
    if (codeRef.current != null) {
      codeRef.current.value = '';
    }
  };

  return (
    <Root>
      <CodeArea ref={codeRef} className="code" defaultValue={defaultValue.trim()} />
      <ButtonArea>
        <Button variant="primary">어셈블!</Button>{' '}
        <Button variant="secondary" onClick={onClickErase}>지우기</Button>{' '}
        <Button variant="danger">시뮬레이터 리셋</Button>
      </ButtonArea>
    </Root>
  );
};

export default AssemblyPage;
