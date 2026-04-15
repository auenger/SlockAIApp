import { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { MainContent } from './components/MainContent';
import { ThreadPanel } from './components/ThreadPanel';
import { TabType, AgentWithRuntime } from './types';
import { useThreadChat } from './lib/useThreadChat';
import { useChannel } from './lib/useChannel';
import { useAgentStatus } from './lib/useAgentStatus';
import { useResizable } from './lib/useResizable';
import { useTasks } from './lib/useTasks';
import { deleteAgent, invoke } from './lib/ipc';

export default function App() {
  const [activeChannel, setActiveChannel] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('CHAT');
  const [isThreadOpen, setIsThreadOpen] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState<AgentWithRuntime | null>(null);
  const [activeThreadId, setActiveThreadId] = useState<string | null>(null);

  // Thread chat hook (shared between Sidebar and MainContent — SINGLE instance)
  const { threads, loadAllThreads, createNewThread, selectThread, activeThread, send: sendThreadMessage, removeThread, renameThreadAction, isStreaming: threadIsStreaming, isThinking: threadIsThinking, streamingText: threadStreamingText, clearActive: clearActiveThread } = useThreadChat();

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

  // Task state — track incomplete tasks for sidebar badge
  const { tasks: allTasks } = useTasks();
  const incompleteTaskCount = allTasks.filter(t =>
    t.status !== 'done' && t.status !== 'cancelled'
  ).length;

  // Resizable panels
  const sidebarResize = useResizable({ initialWidth: 256, minWidth: 180, maxWidth: 400, edge: 'right' });
  const threadResize = useResizable({ initialWidth: 320, minWidth: 280, maxWidth: 600, edge: 'left' });

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

  // Load all threads on mount (global thread list)
  useEffect(() => {
    loadAllThreads();
  }, [loadAllThreads]);

  /** Handle thread creation: notify sidebar and select the new thread */
  const handleThreadCreated = (threadId: string) => {
    setActiveThreadId(threadId);
    loadAllThreads();
  };

  /** Handle thread selection from sidebar */
  const handleThreadSelect = (threadId: string) => {
    setActiveThreadId(threadId);
    setIsThreadOpen(true);
    // Find the thread to get its agent_id for loading messages
    const threadInfo = threads.find(t => t.id === threadId);
    const agentId = threadInfo?.agent_id;
    if (agentId) {
      // Auto-associate the agent if not already selected
      const agentForThread = allAgents.find(a => a.agent.agent_id === agentId);
      if (agentForThread && agentForThread.agent.agent_id !== selectedAgent?.agent.agent_id) {
        setSelectedAgent(agentForThread);
        setActiveChannel(null);
      }
      selectThread(agentId, threadId);
    }
    setActiveTab('CHAT');
  };

  /** Handle sending a message in ThreadPanel */
  const handleSendThreadMessage = async (message: string) => {
    if (!selectedAgent || !activeThreadId) return;
    await sendThreadMessage(selectedAgent.agent.agent_id, activeThreadId, message);
  };

  /** Handle new thread creation from sidebar */
  const handleCreateThread = async (agentId?: string) => {
    const targetAgentId = agentId || selectedAgent?.agent.agent_id;
    if (!targetAgentId) return;
    try {
      const thread = await createNewThread(targetAgentId, '');
      setActiveThreadId(thread.id);
      setActiveTab('CHAT');
      loadAllThreads();
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
    const threadInfo = threads.find(t => t.id === threadId);
    const agentId = threadInfo?.agent_id || selectedAgent?.agent.agent_id;
    if (!agentId) return;
    try {
      await removeThread(agentId, threadId);
      if (activeThreadId === threadId) {
        setActiveThreadId(null);
        setIsThreadOpen(false);
      }
      loadAllThreads();
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
    clearActiveThread(); // Clear thread state (activeThread, streaming, thinking)
    setIsThreadOpen(false);
    setActiveTab('CHAT');
  };

  /** Handle opening the global Task Board view */
  const handleTaskViewOpen = () => {
    setSelectedAgent(null);
    setActiveChannel(null);
    _clearActiveChannel();
    setActiveThreadId(null);
    clearActiveThread();
    setIsThreadOpen(false);
    setActiveTab('TASKS');
  };

  /** Handle refresh: reload current view data */
  const handleRefresh = async () => {
    if (activeChannel) {
      // Channel mode: reload channel data
      await selectChannel(activeChannel);
      await loadChannels();
    }
    // Always refresh global thread list
    await loadAllThreads();
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
        onCreateThread={() => handleCreateThread()}
        onCreateThreadWithAgent={(agentId: string) => handleCreateThread(agentId)}
        onRenameThread={renameThreadAction}
        channels={channels}
        onCreateChannel={handleCreateChannel}
        agents={allAgents}
        onDeleteChannel={handleDeleteChannel}
        onDeleteThread={handleDeleteThread}
        onDeleteAgent={handleDeleteAgent}
        onRefreshAgents={scan}
        isTaskViewActive={activeTab === 'TASKS'}
        onTaskViewOpen={handleTaskViewOpen}
        incompleteTaskCount={incompleteTaskCount}
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
        onSendChannelMessage={(channelId, message, userName) => sendChannelMessage(channelId, message, allAgents, userName)}
        channelIsStreaming={channelIsStreaming}
        channelIsThinking={channelIsThinking}
        channelStreamingText={channelStreamingText}
        channelAgentStreams={channelAgentStreams}
        onDeleteChannel={handleDeleteChannel}
        onDeleteAgent={handleDeleteAgent}
        onRefresh={handleRefresh}
        onStopSession={handleStopSession}
        threadActiveThread={activeThread}
        threadIsStreaming={threadIsStreaming}
        threadIsThinking={threadIsThinking}
        threadStreamingText={threadStreamingText}
        threadSend={sendThreadMessage}
        threadCreateNewThread={createNewThread}
        threadSelectThread={selectThread}
      />

      {/* Right Thread Panel */}
      <ThreadPanel
        isOpen={isThreadOpen}
        thread={activeThread}
        agent={selectedAgent}
        onSend={handleSendThreadMessage}
        onClose={() => setIsThreadOpen(false)}
        onRenameThread={renameThreadAction}
        isThinking={threadIsThinking}
        isStreaming={threadIsStreaming}
        streamingText={threadStreamingText}
        style={threadResize.style}
        resizeHandleRef={threadResize.handleRef}
      />

    </div>
  );
}
