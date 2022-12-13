import React from 'react';
import { NativeLibContext } from '../context/NativeLibContext';
import { Button, Card } from 'react-bootstrap';
import styled from '@emotion/styled';
import RadixValue from '../components/RadixValue';

interface Props {
  scrollIntoView: number
}

const Root = styled.div`
  margin: .5rem;
  line-height: 1.2;
`;

const Address = styled.span`
  color: gray;
`;

const Value = styled.span`
  color: blue;
`;

export const Disassembly = ({ scrollIntoView }: Props): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);

  React.useEffect(() => {
    if (native.initialized) {
      const addr = native.state.pc.toString();
      const elem = document.getElementById(`disasm-row-${addr}`);
      if (elem != null) {
        elem.scrollIntoView({ block: 'center' });
      }
    }
  }, [native.initialized, native.state.pc, scrollIntoView]);

  if (!native.initialized) {
    return null;
  }

  const arr: JSX.Element[] = [];
  for (const k of native.state.disasmList) {
    arr.push(
      <div key={k} id={`disasm-row-${k}`} className={ k === native.state.pc ? 'highlighted-next' : undefined }>
        <>[<Address><RadixValue value={k} format="hex"/></Address>]</>
        {' '}
        <Value><RadixValue value={native.state.disasm[k][0]} format="hex"/></Value>
        {' '}
        {native.state.disasm[k][1]}
      </div>
    );
  }

  return (
    <Card style={{ display: 'inline-block', overflowY: 'scroll' }} className="code">
      <Root>
        {arr}
      </Root>
    </Card>
  );
};
