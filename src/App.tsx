import { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { MainContent } from './components/MainContent';
import { ThreadPanel } from './components/ThreadPanel';
import { CreateTaskModal, InviteHumanModal } from './components/Modals';
import { TabType, AgentWithRuntime } from './types';
import { useThreadChat } from './lib/useThreadChat';
import { useChannel } from './lib/useChannel';
import { useAgentStatus } from './lib/useAgentStatus';

export default function App() {
  const [activeChannel, setActiveChannel] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('CHAT');
  const [isThreadOpen, setIsThreadOpen] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState<AgentWithRuntime | null>(null);
  const [activeThreadId, setActiveThreadId] = useState<string | null>(null);

  // Modal states for demonstration
  const [isCreateTaskModalOpen, setIsCreateTaskModalOpen] = useState(false);
  const [isInviteModalOpen, setIsInviteModalOpen] = useState(false);

  // Thread chat hook (shared between Sidebar and MainContent)
  const { threads, loadThreads, createNewThread, selectThread } = useThreadChat();

  // Channel hook
  const {
    channels,
    activeChannel: channelData,
    loadChannels,
    create: createChannelAction,
    selectChannel,
    send: sendChannelMessage,
    isStreaming: channelIsStreaming,
    isThinking: channelIsThinking,
    streamingText: channelStreamingText,
    agentStreams: channelAgentStreams,
    clearActive: _clearActiveChannel,
  } = useChannel();

  // Agent status for channel member selection
  const { agents: allAgents } = useAgentStatus();

  // Load channels on mount
  useEffect(() => {
    loadChannels();
  }, [loadChannels]);

  // Load threads when agent changes
  useEffect(() => {
    if (selectedAgent) {
      loadThreads(selectedAgent.agent.agent_id);
      setActiveThreadId(null);
    } else {
      setActiveThreadId(null);
    }
  }, [selectedAgent]);

  /** Handle thread creation: notify sidebar and select the new thread */
  const handleThreadCreated = (threadId: string) => {
    setActiveThreadId(threadId);
    if (selectedAgent) {
      loadThreads(selectedAgent.agent.agent_id);
    }
  };

  /** Handle thread selection from sidebar */
  const handleThreadSelect = (threadId: string) => {
    setActiveThreadId(threadId);
    if (selectedAgent) {
      selectThread(selectedAgent.agent.agent_id, threadId);
    }
    setActiveTab('CHAT');
  };

  /** Handle new thread creation from sidebar */
  const handleCreateThread = async () => {
    if (!selectedAgent) return;
    try {
      const thread = await createNewThread(selectedAgent.agent.agent_id, selectedAgent.agent.name);
      setActiveThreadId(thread.id);
      setActiveTab('CHAT');
    } catch (err) {
      console.error('Failed to create thread:', err);
    }
  };

  /** Handle channel selection from sidebar */
  const handleChannelSelect = (channelId: string) => {
    setActiveChannel(channelId);
    setSelectedAgent(null);
    setActiveThreadId(null);
    selectChannel(channelId);
    setActiveTab('CHAT');
  };

  /** Handle channel creation */
  const handleCreateChannel = async (name: string, memberAgentIds: string[]) => {
    try {
      const channel = await createChannelAction(name, memberAgentIds);
      setActiveChannel(channel.id);
      setActiveTab('CHAT');
    } catch (err) {
      console.error('Failed to create channel:', err);
    }
  };

  return (
    <div className="flex h-screen w-full overflow-hidden bg-brutal-bg">
      {/* Left Sidebar */}
      <Sidebar
        activeChannel={activeChannel ?? ''}
        onChannelSelect={handleChannelSelect}
        selectedAgentId={selectedAgent?.agent.agent_id ?? null}
        onAgentSelect={setSelectedAgent}
        threads={threads}
        activeThreadId={activeThreadId}
        onThreadSelect={handleThreadSelect}
        onCreateThread={handleCreateThread}
        channels={channels}
        onCreateChannel={handleCreateChannel}
        agents={allAgents}
      />

      {/* Main Content Area */}
      <MainContent
        activeTab={activeTab}
        onTabChange={setActiveTab}
        onOpenCreateTask={() => setIsCreateTaskModalOpen(true)}
        onOpenInviteHuman={() => setIsInviteModalOpen(true)}
        selectedAgent={selectedAgent}
        activeThreadId={activeThreadId}
        onThreadCreated={handleThreadCreated}
        activeChannel={activeChannel ? channelData : null}
        allAgents={allAgents}
        onSendChannelMessage={(channelId, message) => sendChannelMessage(channelId, message, allAgents)}
        channelIsStreaming={channelIsStreaming}
        channelIsThinking={channelIsThinking}
        channelStreamingText={channelStreamingText}
        channelAgentStreams={channelAgentStreams}
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
