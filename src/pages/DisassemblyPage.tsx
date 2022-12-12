import React from 'react';
import Registers from '../views/Registers';
import styled from '@emotion/styled';
import { MemoryViewer } from '../views/MemoryViewer';

const Root = styled.div`
  display: flex;
  padding: 1rem;
  height: 100%;
`;

const DisassemblyPage = (): JSX.Element => {
  return (
    <Root>
      <Registers />
      <MemoryViewer />
    </Root>
  );
};

export default DisassemblyPage;
