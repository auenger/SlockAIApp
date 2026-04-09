import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { MainContent } from './components/MainContent';
import { ThreadPanel } from './components/ThreadPanel';
import { CreateTaskModal, InviteHumanModal } from './components/Modals';
import { TabType, AgentWithRuntime } from './types';

export default function App() {
  const [activeChannel, setActiveChannel] = useState('kagent-integrate-sap-ai-core');
  const [activeTab, setActiveTab] = useState<TabType>('TASKS');
  const [isThreadOpen, setIsThreadOpen] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState<AgentWithRuntime | null>(null);

  // Modal states for demonstration
  const [isCreateTaskModalOpen, setIsCreateTaskModalOpen] = useState(false);
  const [isInviteModalOpen, setIsInviteModalOpen] = useState(false);

  return (
    <div className="flex h-screen w-full overflow-hidden bg-brutal-bg">
      {/* Left Sidebar */}
      <Sidebar
        activeChannel={activeChannel}
        onChannelSelect={setActiveChannel}
        selectedAgentId={selectedAgent?.agent.agent_id ?? null}
        onAgentSelect={setSelectedAgent}
      />

      {/* Main Content Area */}
      <MainContent
        activeTab={activeTab}
        onTabChange={setActiveTab}
        onOpenCreateTask={() => setIsCreateTaskModalOpen(true)}
        onOpenInviteHuman={() => setIsInviteModalOpen(true)}
        selectedAgent={selectedAgent}
      />

      {/* Right Thread Panel */}
      <ThreadPanel
        isOpen={isThreadOpen}
        onClose={() => setIsThreadOpen(false)}
      />

      {/* Modals */}
      <CreateTaskModal
        isOpen={isCreateTaskModalOpen}
        onClose={() => setIsCreateTaskModalOpen(false)}
      />
      <InviteHumanModal
        isOpen={isInviteModalOpen}
        onClose={() => setIsInviteModalOpen(false)}
      />

      {/* Floating Demo Controls */}
      <div className="fixed bottom-4 right-4 flex flex-col gap-2 z-50">
        <button
          onClick={() => setIsThreadOpen(!isThreadOpen)}
          className="brutal-btn bg-brutal-cyan text-[10px]"
        >
          Toggle Thread
        </button>
        <button
          onClick={() => setIsCreateTaskModalOpen(true)}
          className="brutal-btn bg-brutal-pink text-white text-[10px]"
        >
          Demo: Create Task Modal
        </button>
        <button
          onClick={() => setIsInviteModalOpen(true)}
          className="brutal-btn bg-purple-400 text-white text-[10px]"
        >
          Demo: Invite Modal
        </button>
      </div>
    </div>
  );
}
