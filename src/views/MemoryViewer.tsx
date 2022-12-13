import React from 'react';
import { NativeLibContext } from '../context/NativeLibContext';
import { Button, Card } from 'react-bootstrap';
import styled from '@emotion/styled';
import RadixValue from '../components/RadixValue';

const Root = styled.div`
  margin: .5rem;
  line-height: 1.2;
`;

const Pager = styled.div`
  display: flex;
  font-family: 'Nanum Gothic', sans-serif;
  margin-bottom: 1rem;
  justify-content: center;
  align-items: center;
`;

const CurrentPage = styled.div`
  display: inline-block;
  flex-grow: 1;
  text-align: center;
  vertical-align: middle;
  user-select: none;
  cursor: pointer;
`;

const Address = styled.span`
  color: gray;
`;

const Value = styled.span`
  color: blue;
`;

const bytesPerRow = 16;
const indices = [...Array(4096 / bytesPerRow).keys()];
const rowIndices = [...Array(bytesPerRow).keys()];

const numberToChar = (val: number): string => {
  if (val >= 32 && val < 127) {
    return String.fromCharCode(val);
  } else {
    return '.';
  }
};

interface IProps {
  initialAddr: number
}

interface IState {
  pageAddr: number
  memory?: Uint8Array
}

export const MemoryViewer = ({ initialAddr }: IProps): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);
  const [state, setState] = React.useState<Readonly<IState>>({
    pageAddr: initialAddr & (~0xfff)
  });

  const canGoBack = state.pageAddr > 0;
  const canGoForward = state.pageAddr < 0xfffff000;
  const pageIdx = (state.pageAddr / 4096) | 0;

  React.useEffect(() => {
    if (native.initialized) {
      const prevArr = state.memory ?? new Uint8Array(4096);
      const next = native.lib.readMemory(pageIdx, prevArr);
      setState(prev => ({ ...prev, memory: next }));
    }
  }, [native, state.pageAddr]);

  const memory = state.memory;
  if (!native.initialized || memory == null) {
    return null;
  }

  const handleGoBack = (): void => {
    setState(prev => ({
      ...prev,
      pageAddr: (prev.pageAddr > 0 ? prev.pageAddr - 0x1000 : prev.pageAddr)
    }));
  };

  const handleGoHome = (): void => {
    setState(prev => ({ ...prev, pageAddr: initialAddr & (~0xfff) }));
  };

  const handleGoForward = (): void => {
    setState(prev => ({
      ...prev,
      pageAddr: (prev.pageAddr < 0xfffff000 ? prev.pageAddr + 0x1000 : prev.pageAddr)
    }));
  };

  return (
    <Card style={{ display: 'inline-block', overflowY: 'scroll' }} className="code">
      <Root>
        <Pager>
          <Button variant="secondary" disabled={!canGoBack} onClick={handleGoBack}>← 이전 페이지</Button>
          <CurrentPage onClick={handleGoHome}>
            {'0x'}
            <RadixValue value={state.pageAddr} format="hex" caps={true} />
            {' ~ 0x'}
            <RadixValue value={state.pageAddr + 0xfff} format="hex" caps={true} />
          </CurrentPage>
          <Button variant="secondary" disabled={!canGoForward} onClick={handleGoForward}>다음 페이지 →</Button>
        </Pager>
        {indices.map(i => (
          <div key={i}>
            <>[<Address><RadixValue value={state.pageAddr + i * bytesPerRow} format="hex"/></Address>]</>
            {' '}
            <Value>
              {rowIndices.map(j => (
                <React.Fragment key={i * bytesPerRow + j}>
                  <RadixValue value={memory[i * bytesPerRow + j]} format={'hex'} digits={2}/>
                  {j % 4 === 3 ? ' ' : ''}
                </React.Fragment>
              ))}
            </Value>
            {rowIndices.map(j => (
              <React.Fragment key={i * bytesPerRow + j}>
                {numberToChar(memory[i * bytesPerRow + j])}
              </React.Fragment>
            ))}
          </div>
        ))}
      </Root>
    </Card>
  );
};
