import React from 'react';
import Registers from '../views/Registers';
import styled from '@emotion/styled';
import { MemoryViewer } from '../views/MemoryViewer';
import { Disassembly } from '../views/Disassembly';
import ButtonGroup from 'react-bootstrap/ButtonGroup';
import Button from 'react-bootstrap/Button';
import { NativeLibContext } from '../context/NativeLibContext';
import Form from 'react-bootstrap/Form';

const Root = styled.div`
  display: flex;
  padding: 1rem;
  height: 100%;
`;

const VerticalAlign = styled.div`
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  margin-left: 1rem;
`;

const Title = styled.div`
  text-align: center;
  margin: .5rem 0;
`;

const Status = styled.div`
  margin-left: auto;
  margin-bottom: 1rem;
`;

interface IState {
  scrollIntoView: number
  useJit: boolean
}

const DisassemblyPage = (): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);
  const [state, setState] = React.useState<Readonly<IState>>({
    scrollIntoView: 0,
    useJit: false
  });

  if (!native.initialized) {
    return null;
  }

  const handleRun = (): void => {
    native.lib.run(state.useJit && native.state.canUseJit);
  };

  const handleStop = (): void => {
    native.lib.stop();
  };

  const handleStep = (): void => {
    native.lib.step();
  };

  const handleChangeJit = (e: React.ChangeEvent<HTMLInputElement>): void => {
    if (native.state.canUseJit) {
      setState(prev => ({ ...prev, useJit: e.target.checked }));
    } else {
      setState(prev => ({ ...prev, useJit: false }));
    }
  };

  const handleScrollIntoView = (): void => {
    setState(prev => ({ ...prev, scrollIntoView: (prev.scrollIntoView + 1) & 0xffff }));
  };

  return (
    <Root>
      <Registers />
      <VerticalAlign>
        <Status>
          <ButtonGroup>
            <Button variant="success" disabled={native.state.running} onClick={handleRun}>▶ 실행</Button>
            <Button variant="danger" disabled={!native.state.running} onClick={handleStop}>■ 정지</Button>
            <Button variant="primary" disabled={native.state.running} onClick={handleStep}>→ 스텝</Button>
            <Button variant="secondary" onClick={handleScrollIntoView}>스크롤 초기화</Button>
          </ButtonGroup>
        </Status>
        <Status>
          <Form.Switch label="JIT 사용" checked={state.useJit} onChange={handleChangeJit} disabled={!native.state.canUseJit} />
        </Status>
        <Disassembly scrollIntoView={state.scrollIntoView} />
      </VerticalAlign>
      <VerticalAlign>
        <Title>데이터 섹션</Title>
        <MemoryViewer initialAddr={0x10008000} />
        <Title>스택</Title>
        <MemoryViewer initialAddr={0x7ffffe40} />
      </VerticalAlign>
    </Root>
  );
};

export default DisassemblyPage;
