import React from 'react';
import styled from '@emotion/styled';
import Card from 'react-bootstrap/Card';
import RadixValue from '../components/RadixValue';
import Dropdown from 'react-bootstrap/Dropdown';
import { NativeLibContext } from '../context/NativeLibContext';
import { EditRegisterValueModal } from './EditRegisterValueModal';

const RootTable = styled.div`
  display: inline-block;
  font-family: 'Nanum Gothic Coding', monospace;
  padding: .5rem;
  min-width: 10.5em;
  overflow-x: visible;
  white-space: nowrap;
  user-select: none;
  line-height: 1.2;
`;

const generateRegisterNames = (): { [k: number]: string } => {
  const ret: { [k: number]: string } = {};
  for (let i = 0; i < 32; i++) {
    if (i === 0) {
      ret[i] = 'r0';
    } else if (i === 1) {
      ret[i] = 'at';
    } else if (i <= 3) {
      ret[i] = `v${i - 2}`;
    } else if (i <= 7) {
      ret[i] = `a${i - 4}`;
    } else if (i <= 15) {
      ret[i] = `t${i - 8}`;
    } else if (i <= 23) {
      ret[i] = `s${i - 16}`;
    } else if (i <= 25) {
      ret[i] = `t${i - 16}`;
    } else if (i <= 27) {
      ret[i] = `k${i - 26}`;
    } else if (i === 28) {
      ret[i] = 'gp';
    } else if (i === 29) {
      ret[i] = 'sp';
    } else if (i === 30) {
      ret[i] = 's8';
    } else if (i === 31) {
      ret[i] = 'ra';
    }
  }
  return ret;
};

interface InnerProps {
  children: JSX.Element
  onClick: React.MouseEventHandler<HTMLSpanElement>
}

const MenuToggle = React.forwardRef<HTMLSpanElement, InnerProps>(({ children, onClick }, ref) => {
  return (
    <span ref={ref} onContextMenu={(e) => {
      e.preventDefault();
      onClick(e);
    }}>
      {children}
    </span>
  );
});

MenuToggle.displayName = 'MenuToggle';

export const registerNames: { [k: number]: string | undefined } = generateRegisterNames();

const Registers = (): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);
  const [format, setFormat] = React.useState('hex');
  const [editRegisterDialogIndex, setEditRegisterDialogIndex] = React.useState(-1);

  const closeEditRegisterDialog = (): void => {
    setEditRegisterDialogIndex(-1);
  };

  if (!native.initialized) { return null; }

  const onSelect = (idx: number, key: any): void => {
    switch (key) {
      case 'edit':
        setEditRegisterDialogIndex((prev) => (prev === -1 ? idx : prev));
        break;
      case 'viewBin':
        setFormat('bin');
        break;
      case 'viewDec':
        setFormat('dec');
        break;
      case 'viewHex':
        setFormat('hex');
        break;
      default:
        break;
    }
  };

  return (
    <>
      <EditRegisterValueModal
        regIndex={editRegisterDialogIndex}
        onHide={closeEditRegisterDialog}
        onSet={(idx, value) => native.lib.editRegister(idx, value)} />
    <Card style={{ display: 'inline-block' }}>
      <RootTable>
          {native.state.regs.map((val, idx) => (
            <Dropdown key={idx} onSelect={onSelect.bind(null, idx)}>
              <Dropdown.Toggle as={MenuToggle}>
                <div key={idx}>
                  <span>
                    R{idx.toString().padEnd(2)}&nbsp;[{registerNames[idx]}] =&nbsp;
                  </span>
                  <RadixValue value={val} format={format}/>
                </div>
              </Dropdown.Toggle>
              <Dropdown.Menu>
                <Dropdown.Item eventKey="edit" disabled={idx === 0}>수정</Dropdown.Item>
                <Dropdown.Divider/>
                <Dropdown.Item eventKey="viewBin">2진수로 표시</Dropdown.Item>
                <Dropdown.Item eventKey="viewDec">10진수로 표시</Dropdown.Item>
                <Dropdown.Item eventKey="viewHex">16진수로 표시</Dropdown.Item>
              </Dropdown.Menu>
            </Dropdown>
          ))}
      </RootTable>
    </Card>
    </>
  );
};

export default Registers;
