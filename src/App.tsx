import { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { MainContent } from './components/MainContent';
import { ThreadPanel } from './components/ThreadPanel';
import { TabType, AgentWithRuntime } from './types';
import { useThreadChat } from './lib/useThreadChat';
import { useChannel } from './lib/useChannel';
import { useAgentStatus } from './lib/useAgentStatus';
import { useResizable } from './lib/useResizable';
import { deleteAgent, invoke } from './lib/ipc';

export default function App() {
  const [activeChannel, setActiveChannel] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('CHAT');
  const [isThreadOpen, setIsThreadOpen] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState<AgentWithRuntime | null>(null);
  const [activeThreadId, setActiveThreadId] = useState<string | null>(null);

  // Thread chat hook (shared between Sidebar and MainContent)
  const { threads, loadThreads, createNewThread, selectThread, activeThread, send: sendThreadMessage, removeThread } = useThreadChat();

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
    remove: _removeChannel,
  } = useChannel();

  // Agent status for channel member selection
  const { agents: allAgents, scan } = useAgentStatus();

  // Resizable panels
  const sidebarResize = useResizable({ initialWidth: 256, minWidth: 180, maxWidth: 400, edge: 'right' });
  const threadResize = useResizable({ initialWidth: 320, minWidth: 240, maxWidth: 560, edge: 'left' });

  // Keep selectedAgent in sync with the latest allAgents data
  // This ensures edits to agent properties (name, icon, etc.) are reflected everywhere
  useEffect(() => {
    if (selectedAgent) {
      const updated = allAgents.find(a => a.agent.agent_id === selectedAgent.agent.agent_id);
      if (updated && updated !== selectedAgent) {
        setSelectedAgent(updated);
      }
    }
  }, [allAgents]);

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
    setIsThreadOpen(true);
    if (selectedAgent) {
      selectThread(selectedAgent.agent.agent_id, threadId);
    }
    setActiveTab('CHAT');
  };

  /** Handle sending a message in ThreadPanel */
  const handleSendThreadMessage = async (message: string) => {
    if (!selectedAgent || !activeThreadId) return;
    await sendThreadMessage(selectedAgent.agent.agent_id, activeThreadId, message);
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

  /** Handle channel deletion */
  const handleDeleteChannel = async (channelId: string) => {
    try {
      await _removeChannel(channelId);
      if (activeChannel === channelId) {
        setActiveChannel(null);
      }
    } catch (err) {
      console.error('Failed to delete channel:', err);
    }
  };

  /** Handle thread deletion */
  const handleDeleteThread = async (threadId: string) => {
    if (!selectedAgent) return;
    try {
      await removeThread(selectedAgent.agent.agent_id, threadId);
      if (activeThreadId === threadId) {
        setActiveThreadId(null);
        setIsThreadOpen(false);
      }
    } catch (err) {
      console.error('Failed to delete thread:', err);
    }
  };

  /** Handle agent deletion */
  const handleDeleteAgent = async (agentId: string) => {
    try {
      await deleteAgent(agentId);
      if (selectedAgent?.agent.agent_id === agentId) {
        setSelectedAgent(null);
        setActiveThreadId(null);
        setIsThreadOpen(false);
      }
      scan();
    } catch (err) {
      console.error('Failed to delete agent:', err);
    }
  };

  /** Handle agent selection with proper state cleanup */
  const handleAgentSelect = (agent: AgentWithRuntime) => {
    setSelectedAgent(agent);
    setActiveChannel(null);
    _clearActiveChannel();
    setActiveThreadId(null);
    setIsThreadOpen(false);
    setActiveTab('CHAT');
  };

  /** Handle refresh: reload current view data */
  const handleRefresh = async () => {
    if (activeChannel) {
      // Channel mode: reload channel data
      await selectChannel(activeChannel);
      await loadChannels();
    } else if (selectedAgent) {
      // Agent/Thread mode: reload threads list
      await loadThreads(selectedAgent.agent.agent_id);
    }
  };

  /** Handle stop session: stop the current agent runtime session */
  const handleStopSession = async () => {
    try {
      await invoke('runtime_session_stop');
    } catch (err) {
      console.error('Failed to stop session:', err);
    }
  };

  return (
    <div className="flex h-screen w-full overflow-hidden bg-brutal-bg">
      {/* Left Sidebar */}
      <Sidebar
        activeChannel={activeChannel ?? ''}
        onChannelSelect={handleChannelSelect}
        selectedAgentId={selectedAgent?.agent.agent_id ?? null}
        onAgentSelect={handleAgentSelect}
        threads={threads}
        activeThreadId={activeThreadId}
        onThreadSelect={handleThreadSelect}
        onCreateThread={handleCreateThread}
        channels={channels}
        onCreateChannel={handleCreateChannel}
        agents={allAgents}
        onDeleteChannel={handleDeleteChannel}
        onDeleteThread={handleDeleteThread}
        onDeleteAgent={handleDeleteAgent}
        onRefreshAgents={scan}
        style={sidebarResize.style}
        resizeHandleRef={sidebarResize.handleRef}
      />

      {/* Main Content Area */}
      <MainContent
        activeTab={activeTab}
        onTabChange={setActiveTab}
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
        onDeleteChannel={handleDeleteChannel}
        onDeleteAgent={handleDeleteAgent}
        onRefresh={handleRefresh}
        onStopSession={handleStopSession}
      />

      {/* Right Thread Panel */}
      <ThreadPanel
        isOpen={isThreadOpen}
        thread={activeThread}
        agent={selectedAgent}
        onSend={handleSendThreadMessage}
        onClose={() => setIsThreadOpen(false)}
        style={threadResize.style}
        resizeHandleRef={threadResize.handleRef}
      />

    </div>
  );
}
