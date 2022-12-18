import React from 'react';
import styled from '@emotion/styled';
import SvgPipeline from '../components/SvgPipeline';
import Registers from '../views/Registers';
import Modal from 'react-bootstrap/Modal';
import { NativeLibContext } from '../context/NativeLibContext';
import Button from 'react-bootstrap/Button';
import { IPipelineDetail } from '../NativeLib';

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

const ImageArea = styled.div`
  flex-grow: 1;
  cursor: auto;
  user-select: none;
  height: 100%;
  width: 100%;
  overflow-x: scroll;
`;

const ImageHolder = styled.div`
  height: 100%;
  overflow: clip;
`;

const setStyle = (e: HTMLElement): void => {
  e.style.cursor = 'pointer';
};

interface IState {
  cycle: number
  showInfo: boolean
  infoTitle: string
  infoContent: string
  pipelineDetailRaw: string
  pipelineDetail: IPipelineDetail | null
}

const PipelinePage = (): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);
  const imageRef = React.useRef<SVGElement>(null);
  const containerRefInner = React.useRef<HTMLDivElement>(null);
  const [state, setState] = React.useState<Readonly<IState>>({
    cycle: 0,
    showInfo: false,
    infoTitle: '',
    infoContent: '',
    pipelineDetailRaw: '',
    pipelineDetail: null
  });

  const handleOnClick = (idx: number): void => {
    if (state.pipelineDetail != null) {
      const info = state.pipelineDetail.nodes[idx];
      const content = (info.value as any as number).toString();
      setState(prev => ({ ...prev, showInfo: true, infoTitle: info.name, infoContent: content }));
    }
  };

  React.useEffect(() => {
    if (native.state == null) {
      setState(prev => ({ ...prev, pipelineDetailRaw: '', pipelineDetail: null }));
    } else if (native.state.pipelineDetail !== state.pipelineDetailRaw) {
      const detail = JSON.parse(native.state.pipelineDetail);
      setState(prev => ({ ...prev, pipelineDetailRaw: native.state.pipelineDetail, pipelineDetail: detail }));
    }
  }, [native.state]);

  React.useEffect(() => {
    if (imageRef.current != null && state.pipelineDetail != null) {
      for (let i = 0; i < state.pipelineDetail.nodes.length; i++) {
        const elem = document.getElementById(state.pipelineDetail.nodes[i].name);
        if (elem != null) {
          setStyle(elem);
          elem.onclick = handleOnClick.bind(null, i);
        }
      }

      for (const stage of ['If', 'Id', 'Ex', 'Mem', 'Wb']) {
        const key = `debugIns${stage}`;
        const elem = document.getElementById(`svg-item-debug-${stage.toLowerCase()}-ins`);
        if (elem == null) {
          continue;
        }

        // @ts-expect-error
        const ins: string | null = state.pipelineDetail[key];
        elem.textContent = ins ?? '(bubble)';
      }
    }
  }, [imageRef.current, state.pipelineDetail]);

  const containerRef = React.useCallback((node: HTMLDivElement) => {
    // @ts-expect-error
    containerRefInner.current = node;

    if (node == null) {
      return;
    }

    const calcWidth = (w: number, h: number): string | null => {
      const newWidth = h / 3080 * 6000;
      if (Math.abs(w - newWidth) < 2) {
        return null;
      } else {
        return `${newWidth}px`;
      }
    };

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        let newWidth: string | null;

        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        if (entry.borderBoxSize) {
          newWidth = calcWidth(
            entry.borderBoxSize[0].inlineSize,
            entry.borderBoxSize[0].blockSize
          );
        } else {
          newWidth = calcWidth(
            entry.contentRect.width,
            entry.contentRect.height
          );
        }

        if (newWidth != null) {
          // @ts-expect-error
          entry.target.style.width = newWidth;
        }
      }
    });

    observer.observe(node);
  }, []);

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
    setState(prev => ({ ...prev, cycle: 0 }));
  };

  return (
    <>
      <Modal show={state.showInfo} onHide={handleHide}>
        <Modal.Header closeButton>
          <Modal.Title>{state.infoTitle}</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          {state.infoContent}
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
              명령만 실행할 수 있습니다.
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
        <ImageArea>
          <ImageHolder ref={containerRef}>
            <SvgPipeline ref={imageRef}/>
          </ImageHolder>
        </ImageArea>
      </Root>
    </>
  );
};

export default PipelinePage;
