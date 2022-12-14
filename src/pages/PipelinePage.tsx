import React from 'react';
import styled from '@emotion/styled';
import SvgPipeline from '../components/SvgPipeline';
import Registers from '../views/Registers';
import Modal from 'react-bootstrap/Modal';
import { NativeLibContext } from '../context/NativeLibContext';
import Button from 'react-bootstrap/Button';

const Root = styled.div`
  padding: 1rem;
  height: 100%;
  display: flex;
  align-items: center;
`;

const Panel = styled.div`;
  overflow-y: auto;
  overflow-x: hidden;
  max-height: 100%;
  flex-shrink: 0;
  padding: 1rem;
  border: 1px solid lightgray;
  border-radius: 5px;
`;

const LongText = styled.div`
  margin: 1rem 0;
  width: 10em;
`;

const TextCycles = styled.div`
  margin: .5rem;
  font-size: 1.1em;
`;

const ButtonHolder = styled.div`
  margin: 1rem;
`;

const PipelineImage = styled(SvgPipeline)`
  flex-shrink: 1;
  cursor: auto;
  user-select: none;
`;

const setStyle = (e: HTMLElement): void => {
  e.style.cursor = 'pointer';
};

interface IState {
  cycle: number
  showInfo: boolean
  infoTitle: string
  infoValue: string
}

const PipelinePage = (): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);
  const ref = React.useRef<SVGElement>();
  const [state, setState] = React.useState<Readonly<IState>>({
    cycle: 0,
    showInfo: false,
    infoTitle: '',
    infoValue: ''
  });

  const handleOnClick = (id: string): void => {
    const info = native.state.pipelineDetail[id];
    setState(prev => ({ ...prev, showInfo: true, infoTitle: info.name, infoValue: info.value }));
  };

  React.useEffect(() => {
    if (ref.current != null && Object.prototype.hasOwnProperty.call(native.state, 'pipelineDetail')) {
      for (const k of native.state.pipelineDetailList) {
        const elem = document.getElementById(k);
        if (elem == null) {
          continue;
        }

        setStyle(elem);
        elem.onclick = handleOnClick.bind(null, k);
      }
    }
  }, [ref.current, native.state]);

  if (!native.initialized) {
    return null;
  }

  const handleHide = (): void => {
    setState(prev => ({ ...prev, showInfo: false }));
  };

  const handleCycle = (): void => {
    native.lib.step();
    setState(prev => ({ ...prev, cycle: prev.cycle + 1 }));
  };

  const handleConvertToPipeline = (): void => {
    native.lib.convertToPipeline();
  };

  return (
    <>
      <Modal show={state.showInfo} onHide={handleHide}>
        <Modal.Header closeButton>
          <Modal.Title>{state.infoTitle}</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          {state.infoValue}
        </Modal.Body>
      </Modal>
      <Root>
        <Panel>
          {!native.state.canUsePipeline && <>
            <Button variant="outline-danger" onClick={handleConvertToPipeline}>파이프라인으로 전환</Button>
            <LongText>
              파이프라인을 보기 위해서는 파이프라인 모델로 전환해야 합니다.
              전환하면
              and/&#8203;or/&#8203;add/&#8203;sub/&#8203;slt/&#8203;lw/&#8203;sw/&#8203;beq/&#8203;j
              명령어만 사용할 수 있습니다.
            </LongText>
          </>}
          {native.state.canUsePipeline && <>
            <TextCycles>사이클: {state.cycle}</TextCycles>
            <ButtonHolder>
              <Button variant="primary" onClick={handleCycle}>→ 스텝</Button>
            </ButtonHolder>
          </>}
          <Registers editable={false}/>
        </Panel>
        <PipelineImage ref={ref}/>
      </Root>
    </>
  );
};

export default PipelinePage;
