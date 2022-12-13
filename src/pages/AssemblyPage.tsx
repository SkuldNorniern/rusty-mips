import React from 'react';
import Button from 'react-bootstrap/Button';
import styled from '@emotion/styled';
import { NativeLibContext } from '../context/NativeLibContext';

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
`;

const StatusText = styled.div`
  word-break: break-word;
`;

const ButtonArea = styled.div`
  flex-shrink: 0;
  margin-left: auto;
  margin-right: 0;
`;

const defaultValue = `
# Recursive fibonacci calculator
# Function signature: int fibonacci(int)
# Also saves the result into $gp as an int array (e.g. $gp = fibonacci(2), $gp + 4 = fibonacci(3), ...)
# Modified from https://gist.github.com/libertylocked/068b118354539a8be992
.text
.globl main
main:
    # Calculate fibonacci upto this number (7)
    ori $a0, $0, 7
    or $s0, $ra, $zero
    jal fibonacci

    # Now we have the answer in $v0
    # NOP here so you can check out register pane
    add $0, $0, $0

    or $ra, $s0, $zero
    # Terminate the program
    jr $ra
fibonacci:
    # Prologue
    addi $sp, $sp, -12
    sw $ra, 8($sp)
    sw $s0, 4($sp)
    sw $s1, 0($sp)
    or $s0, $a0, $zero
    ori $v0, $zero, 1 # return value for terminal condition
    slti $t0, $16, 3
    bne $t0, $0, fibonacciExit # check terminal condition
    addi $a0, $s0, -1 # set args for recursive call to f(n-1)
    jal fibonacci
    or $s1, $v0, $zero # store result of f(n-1) to s1
    addi $a0, $s0, -2 # set args for recursive call to f(n-2)
    jal fibonacci
    add $v0, $s1, $v0 # add result of f(n-1) to it
 fibonacciExit:
    # Save value to memory
    add $t0, $s0, $s0
    add $t0, $t0, $t0  # multiply by 4
    addi $t0, $t0, -8  # align that fibonacci(2) ==> $gp
    add $t0, $gp, $t0
    sw $sp, 0($t0)
    # Epilogue
    lw $ra, 8($sp)
    lw $s0, 4($sp)
    lw $s1, 0($sp)
    addi $sp, $sp, 12
    jr $ra
    ## End of function fibonacci
`;

const AssemblyPage = (): JSX.Element => {
  const native = React.useContext(NativeLibContext);
  const codeRef = React.useRef<HTMLTextAreaElement>(null);
  const [lastStatus, setLastStatus] = React.useState('');

  const onClickErase = (): void => {
    if (codeRef.current != null) {
      codeRef.current.value = '';
      setLastStatus('');
    }
  };

  const onAssemble = (): void => {
    if (codeRef.current == null || !native.initialized) { return; }
    const answer = native.lib.assemble(codeRef.current.value);
    if (answer != null) {
      setLastStatus(answer);
    } else {
      setLastStatus('assemble success');
    }
  };

  const onReset = (): void => {
    if (native.initialized) {
      native.lib.reset();
      setLastStatus('');
    }
  };

  return (
    <Root>
      <CodeArea ref={codeRef} className="code" defaultValue={defaultValue.trim()} />
      <StatusArea>
        <StatusText>
          {lastStatus}
        </StatusText>
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
