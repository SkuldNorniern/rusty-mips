import React from 'react';
import Button from 'react-bootstrap/Button';
import styled from '@emotion/styled';
import { NativeLibContext } from '../context/NativeLibContext';
import { EndianSwitch } from '../views/EndianSwitch';
import { ToastContext } from '../context/ToastContext';
import Dropdown from 'react-bootstrap/Dropdown';
import { TEMPLATE_CODE } from '../components/TemplateCode';

const Root = styled.div`
  display: flex;
  flex-direction: column;
  padding: 1rem;
  height: 100%;
`;

const CodeArea = styled.textarea`
  flex: 1;
  margin-bottom: 1rem;
`;

const StatusArea = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

const TemplateArea = styled.div`
  margin-left: .2rem;
`;

const ButtonArea = styled.div`
  flex-shrink: 0;
  margin-right: 0;
`;

interface IState {
  endian: 'big' | 'little'
  canChangeEndian: boolean
}

const AssemblyPage = (): JSX.Element => {
  const native = React.useContext(NativeLibContext);
  const toast = React.useContext(ToastContext);
  const codeRef = React.useRef<HTMLTextAreaElement>(null);
  const [state, setState] = React.useState<Readonly<IState>>(() => ({
    endian: native.lib.getNativeEndian(),
    canChangeEndian: true
  }));

  const onClickErase = (): void => {
    if (codeRef.current != null) {
      codeRef.current.value = '';
    }
  };

  const onAssemble = (): void => {
    if (codeRef.current == null || !native.initialized) { return; }
    const answer = native.lib.assemble(codeRef.current.value, state.endian);
    if (answer != null) {
      setState(prev => ({ ...prev, canChangeEndian: true }));
      toast.showToast('어셈블러', answer);
    } else {
      setState(prev => ({ ...prev, canChangeEndian: false }));
      toast.showToast('어셈블러', 'assemble successful');
    }
  };

  const onReset = (): void => {
    if (native.initialized) {
      native.lib.reset();
      setState(prev => ({ ...prev, lastStatus: '', canChangeEndian: true }));
    }
  };

  const onSetEndian = (endian: 'big' | 'little'): void => {
    if (native.state.cleanAfterReset && state.canChangeEndian) {
      setState(prev => ({ ...prev, endian }));
    }
  };

  const loadExample = (code: string): void => {
    if (codeRef.current != null) {
      codeRef.current.value = code.trim();
    }
  };

  return (
    <Root>
      <CodeArea ref={codeRef} className="code" defaultValue={''} />
      <StatusArea>
        <TemplateArea>
          <Dropdown>
            <Dropdown.Toggle variant="outline-secondary">
              예제 코드 불러오기
            </Dropdown.Toggle>

            <Dropdown.Menu>
              {TEMPLATE_CODE.map((x, i) => (<React.Fragment key={i}>
                <Dropdown.Item onClick={() => loadExample(x.code)}>
                  {x.name}
                </Dropdown.Item>
              </React.Fragment>))}
            </Dropdown.Menu>
          </Dropdown>
        </TemplateArea>
        <EndianSwitch endian={state.endian} canChange={state.canChangeEndian} onSetEndian={onSetEndian} />
        <ButtonArea>
          <Button variant="primary" onClick={onAssemble}>어셈블!</Button>{' '}
          <Button variant="secondary" onClick={onClickErase}>지우기</Button>{' '}
          <Button variant="danger" onClick={onReset}>시뮬레이터 리셋</Button>
        </ButtonArea>
      </StatusArea>
    </Root>
  );
};

export default AssemblyPage;
