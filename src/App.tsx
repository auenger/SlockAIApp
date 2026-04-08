import { Sidebar } from "./components/layout/Sidebar";
import { MainView } from "./components/layout/MainView";
import { DetailView } from "./components/layout/DetailView";

function App() {
  return (
    <div className="flex h-screen bg-slock-bg text-slock-text">
      <Sidebar />
      <MainView />
      <DetailView />
    </div>
  );
}

export default App;
