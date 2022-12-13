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
  margin-left: auto;
  margin-bottom: 1rem;
`;

const DisassemblyPage = (): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);
  const [scrollIntoView, setScrollIntoView] = React.useState(0);

  if (!native.initialized) {
    return null;
  }

  const handleStep = (): void => {
    native.lib.step();
  };

  const handleScrollIntoView = (): void => {
    setScrollIntoView(prev => (prev + 1) & 0xffff);
  };

  return (
    <Root>
      <Registers />
      <DisassemblyAlign>
        <Status>
          <ButtonGroup>
            <Button variant="success">▶ 실행</Button>
            <Button variant="danger">■ 정지</Button>
            <Button variant="primary" onClick={handleStep}>→ 스텝</Button>
            <Button variant="secondary" onClick={handleScrollIntoView}>스크롤 초기화</Button>
          </ButtonGroup>
        </Status>
        <Disassembly scrollIntoView={scrollIntoView} />
      </DisassemblyAlign>
      <MemoryViewer />
    </Root>
  );
};

export default DisassemblyPage;
