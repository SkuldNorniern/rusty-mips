import React, { ChangeEvent } from 'react';
import Modal from 'react-bootstrap/Modal';
import Alert from 'react-bootstrap/Alert';
import Form from 'react-bootstrap/Form';
import Button from 'react-bootstrap/Button';
import { registerNames } from './Registers';
import styled from '@emotion/styled';

const RadixSelectorGroup = styled.div`
  margin-top: 1rem;
`;

interface Props {
  regIndex: number
  onHide: () => void
  onSet: (idx: number, val: number) => void
}

interface IState {
  regIndex: number
  value: string
  radix: 'bin' | 'dec' | 'hex'
  needsReset: boolean
}

const defaultState = (): Readonly<IState> => {
  return {
    regIndex: 0,
    value: '',
    radix: 'dec',
    needsReset: false
  };
};

export const EditRegisterValueModal = ({ regIndex, onHide, onSet }: Props): JSX.Element => {
  const [state, setState] = React.useState<Readonly<IState>>(defaultState());

  React.useEffect(() => {
    if (regIndex > 0 && regIndex !== state.regIndex) {
      setState(prev => {
        if (prev.needsReset) {
          return { ...defaultState(), regIndex };
        } else if (regIndex > 0 && regIndex !== prev.regIndex) {
          return { ...prev, regIndex };
        } else {
          return prev;
        }
      });
    } else if (regIndex <= 0 && !state.needsReset) {
      setState(prev => ({ ...prev, needsReset: true }));
    }
  }, [regIndex, state]);

  const parseValue = (): number => {
    switch (state.radix) {
      case 'bin':
        return Number.parseInt(state.value, 2);
      case 'dec':
        return Number.parseInt(state.value, 10);
      case 'hex':
        if (/^[0-9A-Fa-f]*$/.test(state.value)) {
          return Number.parseInt(state.value, 16);
        } else {
          return NaN;
        }
      default:
        return NaN;
    }
  };

  const handleChange = (e: ChangeEvent<HTMLInputElement>): void => {
    const value = e.target.value;
    setState(prev => ({ ...prev, value }));
  };

  const handleEdit = (): void => {
    const value = parseValue();
    if (!Number.isNaN(value) && state.regIndex > 0) {
      onSet(state.regIndex, value);
      onHide();
    }
  };

  return (
    <Modal show={regIndex > 0} onHide={onHide}>
      <Modal.Header closeButton>
        <Modal.Title>레지스터 <code>${registerNames[state.regIndex]}</code>를 수정</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        {state.value !== '' && Number.isNaN(parseValue()) && (<Alert variant="danger">올바른 값을 입력해주세요!</Alert>)}
        <input placeholder="새로운 값" value={state.value} onChange={handleChange}/>
        <RadixSelectorGroup>
          <Form.Check
            inline
            label="2진수"
            type="radio"
            id="EditRegisterValueModalRadio-bin"
            name="EditRegisterValueModalRadio"
            onChange={() => setState(prev => ({ ...prev, radix: 'bin' }))}
            checked={state.radix === 'bin'} />
          <Form.Check
            inline
            label="10진수"
            type="radio"
            id="EditRegisterValueModalRadio-dec"
            name="EditRegisterValueModalRadio"
            onChange={() => setState(prev => ({ ...prev, radix: 'dec' }))}
            checked={state.radix === 'dec'} />
          <Form.Check
            inline
            label="16진수"
            type="radio"
            id="EditRegisterValueModalRadio-hex"
            name="EditRegisterValueModalRadio"
            onChange={() => setState(prev => ({ ...prev, radix: 'hex' }))}
            checked={state.radix === 'hex'} />
        </RadixSelectorGroup>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" onClick={onHide}>
          취소
        </Button>
        <Button variant="primary" onClick={handleEdit}>
          수정
        </Button>
      </Modal.Footer>
    </Modal>
  );
};
