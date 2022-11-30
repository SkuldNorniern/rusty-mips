import React from 'react';
import styled from '@emotion/styled';
import EditableRegisterValue from '../components/EditableRegisterValue';

const Root = styled.div`
  display: inline-block;
  box-sizing: border-box;
  padding: 1rem;
  font-family: 'Nanum Gothic Coding', monospace;
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

const registerNames: { [k: number]: string | undefined } = generateRegisterNames();

const Disassembly = (): JSX.Element => {
  const regs: number[] = [];
  const [format, setFormat] = React.useState('hex');

  for (let i = 0; i < 32; i++) {
    regs[i] = i;
  }

  const changeFormat = (type: string): void => {
    setFormat(type);
  };

  return (
    <Root>
      <table>
        <tbody>
          {regs.map((val, idx) => (
            <tr key={idx}>
              <td>
                R{idx}&nbsp;
              </td>
              <td>
                [{registerNames[idx]}] =&nbsp;
              </td>
              <td>
                <EditableRegisterValue value={val} format={format} changeFormat={changeFormat}/>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </Root>
  );
};

export default Disassembly;
