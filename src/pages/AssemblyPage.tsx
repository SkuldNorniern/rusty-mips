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
  return (
    <Root>
      <CodeArea className="code" defaultValue={defaultValue.trim()} />
      <ButtonArea>
        <Button variant="primary">Assemble!</Button>{' '}
        <Button variant="secondary">Clear</Button>{' '}
        <Button variant="danger">Reset simulator</Button>
      </ButtonArea>
    </Root>
  );
};

export default AssemblyPage;
