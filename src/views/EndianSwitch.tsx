import React from 'react';
import Form from 'react-bootstrap/Form';
import { NativeLibContext } from '../context/NativeLibContext';
import styled from '@emotion/styled';

const Root = styled.div`
  vertical-align: middle;
  flex-shrink: 0;
  margin-left: auto;
`;

const Text = styled.div`
  display: inline-block;
  margin-right: 1rem;
  margin-left: .3rem;
`;

interface IProps {
  endian: 'big' | 'little'
  onSetEndian: (endian: 'big' | 'little') => void
  canChange: boolean
}

export const EndianSwitch = ({ endian, onSetEndian, canChange }: IProps): JSX.Element | null => {
  const native = React.useContext(NativeLibContext);

  if (!native.initialized) {
    return null;
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    let preferred: 'big' | 'little';
    if (e.target.checked) {
      preferred = 'little';
    } else {
      preferred = 'big';
    }

    onSetEndian(preferred);
  };

  return (
    <Root>
      <Text>빅 엔디안</Text>
      <Form.Switch
        style={{ display: 'inline-block' }}
        checked={endian === 'little'}
        onChange={handleChange}
        disabled={!native.state.cleanAfterReset && canChange} />
      <Text>리틀 엔디안</Text>
    </Root>
  );
};
