import React from 'react';
import Tabs from 'react-bootstrap/Tabs';
import Tab from 'react-bootstrap/Tab';
import DisassemblyPage from './pages/DisassemblyPage';
import AssemblyPage from './pages/AssemblyPage';
import PipelinePage from './pages/PipelinePage';
import { NativeLibProvider } from './context/NativeLibContext';

enum TabName {
  assembly,
  disassembly,
  pipeline,
}

const App = (): JSX.Element => {
  return (
    <NativeLibProvider>
      <div className="fullscreen-tab-root">
        <Tabs>
          <Tab eventKey={TabName.assembly} title="어셈블리">
            <AssemblyPage/>
          </Tab>
          <Tab eventKey={TabName.disassembly} title="디스어셈블리">
            <DisassemblyPage />
          </Tab>
          <Tab eventKey={TabName.pipeline} title="파이프라인">
            <PipelinePage />
          </Tab>
        </Tabs>
      </div>
    </NativeLibProvider>
  );
};

export default App;
