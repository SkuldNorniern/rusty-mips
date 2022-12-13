import React from 'react';
import Registers from '../views/Registers';
import styled from '@emotion/styled';
import { MemoryViewer } from '../views/MemoryViewer';
import { Disassembly } from '../views/Disassembly';
import ButtonGroup from 'react-bootstrap/ButtonGroup';
import Button from 'react-bootstrap/Button';
import { NativeLibContext } from '../context/NativeLibContext';

const Root = styled.div`
  display: flex;
  padding: 1rem;
  height: 100%;
`;

const DisassemblyAlign = styled.div`
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  margin-left: 1rem;
`;

const Status = styled.div`
  display: flex;
  margin-bottom: 1rem;
`;

const Info = styled.div`
  display: block;
  margin-right: auto;
`;

const DisassemblyPage = (): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);

  if (!native.initialized) {
    return null;
  }

  return (
    <Root>
      <Registers />
      <DisassemblyAlign>
        <Status>
          <Info>{native.state.running ? '실행중' : '정지됨'}</Info>
          <ButtonGroup>
            <Button variant="success">▶ 실행</Button>
            <Button variant="danger">■ 정지</Button>
            <Button variant="primary">→ 스텝</Button>
          </ButtonGroup>
        </Status>
        <Disassembly />
      </DisassemblyAlign>
      <MemoryViewer />
    </Root>
  );
};

export default DisassemblyPage;
