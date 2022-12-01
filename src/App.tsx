import React from 'react';
import Tabs from 'react-bootstrap/Tabs';
import Tab from 'react-bootstrap/Tab';
import DisassemblyPage from './pages/DisassemblyPage';
import AssemblyPage from './pages/AssemblyPage';

enum TabName {
  assembly,
  disassembly,
}

const App = (): JSX.Element => {
  return (
    <div className="fullscreen-tab-root">
      <Tabs>
        <Tab eventKey={TabName.assembly} title="Assembly">
          <AssemblyPage/>
        </Tab>
        <Tab eventKey={TabName.disassembly} title="Disassembly">
          <DisassemblyPage />
        </Tab>
      </Tabs>
    </div>
  );
};

export default App;
