import React from 'react';
import Registers from '../views/Registers';
import styled from '@emotion/styled';

const Root = styled.div`
  padding: 1rem;
  height: 100%;
`;

const DisassemblyPage = (): JSX.Element => {
  return (
    <Root>
      <Registers/>
    </Root>
  );
};

export default DisassemblyPage;
