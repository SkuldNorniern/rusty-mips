import React from 'react';
import Tabs from 'react-bootstrap/Tabs';
import Tab from 'react-bootstrap/Tab';
import DisassemblyPage from './pages/DisassemblyPage';

enum TabName {
  assembly,
  disassembly,
}

const App = (): JSX.Element => {
  return (
    <Tabs>
      <Tab eventKey={TabName.assembly} title="Assembly">
      </Tab>
      <Tab eventKey={TabName.disassembly} title="Disassembly">
        <DisassemblyPage />
      </Tab>
    </Tabs>
  );
};

export default App;
