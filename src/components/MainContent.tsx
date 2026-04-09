import React, { useState, useRef, useEffect } from 'react';
import {
  MessageSquare,
  CheckSquare,
  Folder,
  Zap,
  Activity,
  User,
  Plus,
  Trash2,
  Users,
  ChevronRight,
  ChevronLeft,
  Bot,
  Circle,
  Square,
  RotateCcw,
  Copy,
  Send,
  Image as ImageIcon,
  Terminal,
  Search,
  Code,
  Globe,
  Database,
  Hash,
  AtSign,
  FileText,
  Loader2,
} from 'lucide-react';
import { cn } from '../lib/utils';
import { TabType, Task, Message, AgentWithRuntime, Channel, ChannelMessage } from '../types';
import { getRuntimeStatusColor, getRuntimeStatusLabel } from '../lib/useAgentStatus';
import { useAgentProfile } from '../lib/useAgentProfile';
import { useThreadChat } from '../lib/useThreadChat';
import { useWorkspace } from '../lib/useWorkspace';
import { MentionAutocomplete, renderMentionText } from './MentionAutocomplete';
import type { AgentStreamState } from '../lib/useChannel';

// ---------------------------------------------------------------------------
// Agent color palette for distinguishing multi-Agent replies
// ---------------------------------------------------------------------------

const AGENT_COLORS = [
  'bg-brutal-cyan',
  'bg-brutal-pink',
  'bg-brutal-yellow',
  'bg-purple-400',
  'bg-brutal-green',
  'bg-orange-400',
  'bg-teal-400',
  'bg-red-400',
];

function getAgentColor(index: number): string {
  return AGENT_COLORS[index % AGENT_COLORS.length];
}

/** Format file size in bytes to human-readable string */
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

// ---------------------------------------------------------------------------
// Agent Streaming Bubble
// ---------------------------------------------------------------------------

interface AgentStreamBubbleProps {
  stream: AgentStreamState;
  agentInfo?: AgentWithRuntime;
  colorIndex: number;
}

const AgentStreamBubble: React.FC<AgentStreamBubbleProps> = ({
  stream,
  agentInfo,
  colorIndex,
}) => {
  const agentName = agentInfo?.agent.name || stream.agent_id;
  const agentEmoji = agentInfo?.agent.emoji?.charAt(0) || 'A';
  const bgColor = getAgentColor(colorIndex);

  return (
    <div className="flex gap-3 px-2">
      <div className={cn(
        "w-8 h-8 brutal-border flex items-center justify-center shrink-0 font-black",
        bgColor
      )}>
        {agentEmoji}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className={cn("font-black text-xs", bgColor === 'bg-brutal-yellow' ? 'text-black' : 'text-black')}>
            {agentName}
          </span>
          {stream.thinking ? (
            <span className="text-[8px] text-gray-500 uppercase italic">Thinking...</span>
          ) : stream.streaming ? (
            <span className="text-[8px] text-gray-500 uppercase italic">Streaming...</span>
          ) : stream.done ? (
            <span className="text-[8px] text-brutal-green uppercase italic">Done</span>
          ) : null}
          {stream.total_agents > 1 && (
            <span className="text-[8px] text-gray-400">
              ({stream.agent_index + 1}/{stream.total_agents})
            </span>
          )}
        </div>
        {stream.thinking && !stream.text ? (
          <div className="h-4 bg-gray-200 w-2/3 brutal-border-b animate-pulse" />
        ) : (
          <div className="text-sm leading-relaxed whitespace-pre-wrap">
            {stream.text}
            {stream.streaming && (
              <span className="inline-block w-1.5 h-4 bg-brutal-cyan ml-0.5 animate-pulse" />
            )}
          </div>
        )}
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// MainContent Props
// ---------------------------------------------------------------------------

interface MainContentProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
  onOpenCreateTask?: () => void;
  onOpenInviteHuman?: () => void;
  /** Currently selected agent (null = no agent selected) */
  selectedAgent?: AgentWithRuntime | null;
  /** Active thread ID from sidebar */
  activeThreadId?: string | null;
  /** Callback when a new thread is created */
  onThreadCreated?: (threadId: string) => void;
  /** Active channel data (when a channel is selected) */
  activeChannel?: Channel | null;
  /** All agents (for resolving channel member names) */
  allAgents?: AgentWithRuntime[];
  /** Send a message in a channel */
  onSendChannelMessage?: (channelId: string, message: string) => Promise<void>;
  /** Whether a channel message is streaming */
  channelIsStreaming?: boolean;
  /** Whether the channel agent is thinking */
  channelIsThinking?: boolean;
  /** Buffered streaming text for channel */
  channelStreamingText?: string;
  /** Per-agent streaming states for multi-Agent responses */
  channelAgentStreams?: AgentStreamState[];
}

export const MainContent: React.FC<MainContentProps> = ({
  activeTab,
  onTabChange,
  onOpenCreateTask,
  selectedAgent,
  activeThreadId,
  onThreadCreated,
  activeChannel,
  allAgents = [],
  onSendChannelMessage,
  channelIsStreaming = false,
  channelIsThinking = false,
  channelStreamingText = '',
  channelAgentStreams = [],
}) => {
  const [taskFilter, setTaskFilter] = useState('All');
  const [inputValue, setInputValue] = useState('');
  const [logs, setLogs] = useState<{ time: string; status: string; text?: string; color?: string }[]>([
    { time: '08:27:17 AM', status: 'Thinking', color: 'text-brutal-yellow' },
    { time: '08:27:17 AM', status: 'Idle', color: 'text-brutal-green', text: 'Idle' },
    { time: '08:30:09 AM', status: 'Working', color: 'text-brutal-cyan', text: 'Message received' },
  ]);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Whether we're in channel mode (vs agent/thread mode)
  const isChannelMode = !!activeChannel;

  // Build a lookup map for agent info (used by channel messages)
  const agentMap = new Map(allAgents.map((a) => [a.agent.agent_id, a]));

  // Track color assignments for agents
  const agentColorMap = new Map<string, number>();
  let colorIdx = 0;
  for (const member of activeChannel?.members || []) {
    agentColorMap.set(member.agent_id, colorIdx++);
  }

  // Thread chat hook
  const {
    activeThread,
    isStreaming,
    isThinking,
    streamingText,
    createNewThread,
    selectThread,
    send,
  } = useThreadChat();

  const addLog = (status: string, text?: string, color?: string) => {
    const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    setLogs(prev => [...prev, { time, status, text, color }]);
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [activeThread?.messages, activeChannel?.messages, isThinking, streamingText, channelIsThinking, channelStreamingText, channelAgentStreams]);

  // When activeThreadId changes (from sidebar), load the thread
  useEffect(() => {
    if (activeThreadId && selectedAgent) {
      selectThread(selectedAgent.agent.agent_id, activeThreadId);
    }
  }, [activeThreadId, selectedAgent]);

  // Profile data loading
  const { data: profileData, loading: profileLoading, loadProfile } = useAgentProfile();

  // Load profile when selectedAgent changes
  useEffect(() => {
    if (selectedAgent?.agent.agent_id) {
      loadProfile(selectedAgent.agent.agent_id);
    }
  }, [selectedAgent?.agent.agent_id, loadProfile]);

  // Workspace browser hook
  const {
    entries,
    selectedFile,
    workspacePath,
    currentPath,
    loadingDir,
    loadingFile,
    error: workspaceError,
    loadDir,
    loadFile,
    navigateInto,
    navigateUp,
  } = useWorkspace();

  // Load workspace when selectedAgent changes or tab becomes WORKSPACE
  useEffect(() => {
    if (selectedAgent?.agent.agent_id && activeTab === 'WORKSPACE') {
      loadDir(selectedAgent.agent.agent_id);
    }
  }, [selectedAgent?.agent.agent_id, activeTab, loadDir]);

  const handleSendMessage = async () => {
    if (!inputValue.trim() || !selectedAgent) return;
    if (isStreaming || isThinking) return;

    const agentId = selectedAgent.agent.agent_id;
    let threadId = activeThread?.id;

    // If no active thread, create one
    if (!threadId) {
      try {
        addLog('Working', 'Creating new thread...', 'text-brutal-cyan');
        const newThread = await createNewThread(agentId, selectedAgent.agent.name);
        threadId = newThread.id;
        onThreadCreated?.(threadId);
      } catch (err) {
        addLog('Error', 'Failed to create thread', 'text-brutal-pink');
        return;
      }
    }

    const userInput = inputValue;
    setInputValue('');
    addLog('Working', 'Message sent', 'text-brutal-cyan');
    addLog('Thinking', undefined, 'text-brutal-yellow');

    try {
      await send(agentId, threadId, userInput);
      addLog('Output', `Responded to: ${userInput.substring(0, 20)}...`);
    } catch (error) {
      console.error("Agent Error:", error);
      addLog('Error', 'Failed to generate response', 'text-brutal-pink');
    } finally {
      addLog('Idle', 'Task completed', 'text-brutal-green');
    }
  };

  // Convert thread messages to display format
  const displayMessages: Message[] = (() => {
    if (!activeThread) return [];
    return activeThread.messages.map((msg) => ({
      id: msg.id,
      sender: {
        name: msg.role === 'user' ? 'You' : (selectedAgent?.agent.name || 'Agent'),
        avatar: msg.role === 'user' ? 'U' : (selectedAgent?.agent.emoji?.charAt(0) || 'A'),
        isAgent: msg.role === 'agent',
      },
      content: msg.content,
      timestamp: new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    }));
  })();

  // Convert channel messages to display format with per-agent colors
  const channelDisplayMessages: (Message & { agentColor?: string; agentEmoji?: string })[] = (() => {
    if (!activeChannel) return [];
    return activeChannel.messages.map((msg: ChannelMessage) => {
      const isAgent = msg.sender_type === 'agent';
      const agentInfo = isAgent ? agentMap.get(msg.sender_id) : null;
      const cIdx = isAgent ? (agentColorMap.get(msg.sender_id) ?? 0) : 0;
      return {
        id: msg.id,
        sender: {
          name: isAgent ? (agentInfo?.agent.name || msg.sender_id) : 'You',
          avatar: isAgent ? (agentInfo?.agent.emoji?.charAt(0) || 'A') : 'U',
          isAgent,
        },
        content: msg.content,
        timestamp: new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        agentColor: isAgent ? getAgentColor(cIdx) : undefined,
        agentEmoji: isAgent ? agentInfo?.agent.emoji : undefined,
      };
    });
  })();

  /** Handle sending a message in channel mode */
  const handleSendChannelMessage = async () => {
    if (!inputValue.trim() || !activeChannel || !onSendChannelMessage) return;
    if (channelIsStreaming || channelIsThinking) return;

    const userInput = inputValue;
    setInputValue('');
    addLog('Working', 'Channel message sent', 'text-brutal-cyan');
    addLog('Thinking', undefined, 'text-brutal-yellow');

    try {
      await onSendChannelMessage(activeChannel.id, userInput);
      addLog('Output', `Channel responded to: ${userInput.substring(0, 20)}...`);
    } catch (error) {
      console.error("Channel Error:", error);
      addLog('Error', 'Failed to generate channel response', 'text-brutal-pink');
    } finally {
      addLog('Idle', 'Channel task completed', 'text-brutal-green');
    }
  };

  const agentName = selectedAgent?.agent.name || 'Agent';
  const canSend = isChannelMode
    ? inputValue.trim() && !channelIsStreaming && !channelIsThinking
    : inputValue.trim() && !isStreaming && !isThinking;

  // Determine header info based on mode
  const headerTitle = isChannelMode
    ? activeChannel!.name
    : (selectedAgent ? selectedAgent.agent.name : 'Select an Agent');
  const headerSubtitle = isChannelMode
    ? `${activeChannel!.members.length} member${activeChannel!.members.length !== 1 ? 's' : ''}`
    : (selectedAgent ? selectedAgent.agent.agent_id : '');

  const tabs: { id: TabType; label: string; icon: React.ElementType }[] = [
    { id: 'CHAT', label: 'CHAT', icon: MessageSquare },
    { id: 'TASKS', label: 'TASKS', icon: CheckSquare },
    { id: 'WORKSPACE', label: 'WORKSPACE', icon: Folder },
    { id: 'SKILLS', label: 'SKILLS', icon: Zap },
    { id: 'ACTIVITY', label: 'ACTIVITY', icon: Activity },
    { id: 'PROFILE', label: 'PROFILE', icon: User },
  ];

  const tasks: Task[] = [
    { id: 2, title: 'test', status: 'TODO' },
    { id: 1, title: '总结oauth2在kagent中的流程', status: 'IN PROGRESS', assignee: '克劳德' },
  ];

  // Get channel members as AgentWithRuntime[] for MentionAutocomplete
  const channelMembers = isChannelMode
    ? activeChannel!.members
        .map((m) => agentMap.get(m.agent_id))
        .filter((a): a is AgentWithRuntime => a !== undefined)
    : [];

  return (
    <div className="flex-1 flex flex-col min-w-0 bg-white">
      {/* Top Header */}
      <div className="p-3 brutal-border-b flex items-center justify-between bg-white">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 brutal-border bg-brutal-cyan flex items-center justify-center">
            {isChannelMode ? (
              <Hash size={24} />
            ) : selectedAgent ? (
              <span className="text-xl">{selectedAgent.agent.emoji}</span>
            ) : (
              <Bot size={24} />
            )}
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="font-black text-lg">{headerTitle}</h2>
              <span className="text-[10px] text-gray-500 font-medium">
                {headerSubtitle}
              </span>
            </div>
            {isChannelMode ? (
              <div className="flex items-center gap-1">
                {activeChannel!.members.slice(0, 5).map((member) => {
                  const memberAgent = agentMap.get(member.agent_id);
                  return (
                    <div
                      key={member.agent_id}
                      className="w-5 h-5 brutal-border bg-brutal-cyan flex items-center justify-center text-[10px]"
                      title={memberAgent?.agent.name || member.agent_id}
                    >
                      {memberAgent?.agent.emoji?.charAt(0) || '?'}
                    </div>
                  );
                })}
                {activeChannel!.members.length > 5 && (
                  <span className="text-[9px] text-gray-500">
                    +{activeChannel!.members.length - 5}
                  </span>
                )}
              </div>
            ) : (
              <div className="flex items-center gap-1.5">
                {selectedAgent ? (
                  <>
                    <Circle
                      size={8}
                      fill={getRuntimeStatusColor(selectedAgent.runtime_status)}
                      className="shrink-0"
                    />
                    <span className="text-[10px] font-bold uppercase tracking-tighter">
                      {getRuntimeStatusLabel(selectedAgent.runtime_status)}
                    </span>
                    {selectedAgent.runtime_version && (
                      <span className="text-[10px] text-gray-400 ml-1">
                        v{selectedAgent.runtime_version}
                      </span>
                    )}
                  </>
                ) : (
                  <>
                    <Circle size={8} fill="#9CA3AF" className="text-gray-400" />
                    <span className="text-[10px] font-bold uppercase tracking-tighter text-gray-400">
                      No Agent Selected
                    </span>
                  </>
                )}
              </div>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button className="p-2 brutal-border hover:bg-gray-100">
            <Square size={18} />
          </button>
          <button className="p-2 brutal-border hover:bg-gray-100">
            <RotateCcw size={18} />
          </button>
          <button className="p-2 brutal-border bg-brutal-yellow hover:bg-yellow-400">
            <Trash2 size={18} />
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex brutal-border-b bg-gray-50">
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={cn(
              "px-4 py-2 font-black text-xs flex items-center gap-2 brutal-border-r transition-all",
              activeTab === tab.id ? "bg-brutal-yellow brutal-border-b-0 translate-y-[1px]" : "hover:bg-gray-200"
            )}
          >
            <tab.icon size={14} />
            {tab.label}
          </button>
        ))}
        <div className="flex-1" />
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-y-auto p-4 bg-brutal-bg/20">
        {activeTab === 'TASKS' && (
          <div className="space-y-4">
            {/* Task Filters */}
            <div className="flex items-center justify-between">
              <div className="flex gap-1">
                {['All 2', 'Todo 1', 'In Progress 1', 'In Review', 'Done'].map(f => (
                  <button
                    key={f}
                    onClick={() => setTaskFilter(f.split(' ')[0])}
                    className={cn(
                      "px-2 py-0.5 brutal-border text-[10px] font-black uppercase",
                      taskFilter === f.split(' ')[0] ? "bg-brutal-yellow" : "bg-white hover:bg-gray-100"
                    )}
                  >
                    {f}
                  </button>
                ))}
              </div>
              <button
                onClick={onOpenCreateTask}
                className="brutal-btn bg-brutal-pink text-white text-xs flex items-center gap-1"
              >
                <Plus size={14} /> New Task
              </button>
            </div>

            {/* Task List */}
            <div className="space-y-2">
              {tasks
                .filter(t => taskFilter === 'All' || t.status.startsWith(taskFilter))
                .map(task => (
                <div key={task.id} className="brutal-card flex items-center justify-between py-2 px-3">
                  <div className="flex items-center gap-3">
                    <ChevronRight size={14} className="text-gray-400" />
                    <span className="text-gray-400 font-mono text-xs">#{task.id}</span>
                    <span className={cn(
                      "px-1.5 py-0.5 brutal-border text-[8px] font-black",
                      task.status === 'TODO' ? "bg-brutal-yellow" : "bg-brutal-cyan"
                    )}>
                      {task.status}
                    </span>
                    <span className="font-bold text-sm">{task.title}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {task.assignee && (
                      <div className="flex items-center gap-1 px-1.5 py-0.5 brutal-border bg-gray-50 text-[10px] font-bold">
                        <span className="text-gray-500">@</span>{task.assignee}
                      </div>
                    )}
                    <div className="flex gap-1">
                      <button className="p-1 brutal-border bg-brutal-cyan hover:bg-cyan-400">
                        <Users size={14} />
                      </button>
                      <button className="p-1 brutal-border hover:bg-gray-100">
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'WORKSPACE' && (
          <div className="h-full flex flex-col">
            {!selectedAgent ? (
              <div className="flex-1 flex flex-col items-center justify-center text-gray-400">
                <Folder size={48} strokeWidth={1} className="mb-2 opacity-20" />
                <span className="text-sm italic">Select an agent to view workspace</span>
              </div>
            ) : (
              <>
                <div className="flex items-center gap-2 mb-4 text-[10px] font-mono text-gray-500 bg-white brutal-border p-1">
                  <button
                    onClick={() => navigateUp(selectedAgent.agent.agent_id)}
                    disabled={!currentPath}
                    className={cn(
                      "p-0.5 brutal-border hover:bg-gray-100",
                      !currentPath && "opacity-30 cursor-not-allowed"
                    )}
                  >
                    <ChevronLeft size={10} />
                  </button>
                  <span className="truncate flex-1">{workspacePath}{currentPath}</span>
                  <button
                    onClick={() => loadDir(selectedAgent.agent.agent_id, currentPath || undefined)}
                    className="p-0.5 brutal-border hover:bg-gray-100"
                  >
                    <RotateCcw size={10} className={loadingDir ? "animate-spin" : ""} />
                  </button>
                  <button
                    onClick={() => navigator.clipboard.writeText(workspacePath + currentPath)}
                    className="p-0.5 brutal-border hover:bg-gray-100"
                  >
                    <Copy size={10} />
                  </button>
                </div>
                <div className="flex-1 flex brutal-border bg-white overflow-hidden">
                  <div className="w-64 brutal-border-r p-2 space-y-1 overflow-y-auto">
                    <div className="flex items-center justify-between px-1 mb-2">
                      <span className="text-[10px] font-black uppercase text-gray-500">
                        {currentPath ? currentPath.split("/").pop() : "Workspace"}
                      </span>
                      <span className="text-[10px] text-gray-400">{entries.length} items</span>
                    </div>
                    {loadingDir ? (
                      <div className="flex items-center justify-center py-8">
                        <Loader2 size={24} className="animate-spin text-gray-400" />
                      </div>
                    ) : workspaceError ? (
                      <div className="text-xs text-red-500 p-2">{workspaceError}</div>
                    ) : (
                      <div className="space-y-1">
                        {entries.map((entry) => (
                          <div
                            key={entry.name}
                            onClick={() => {
                              if (entry.is_dir) {
                                navigateInto(selectedAgent.agent.agent_id, entry.name);
                              } else {
                                const filePath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
                                loadFile(selectedAgent.agent.agent_id, filePath);
                              }
                            }}
                            className={cn(
                              "flex items-center gap-2 px-2 py-1 text-sm font-bold cursor-pointer transition-colors",
                              selectedFile?.name === entry.name ? "bg-brutal-bg brutal-border-l-4 border-l-black" : "hover:bg-gray-100"
                            )}
                          >
                            {entry.is_dir ? (
                              <>
                                <ChevronRight size={14} className="text-gray-400" />
                                <Folder size={14} className="text-brutal-yellow fill-brutal-yellow" />
                              </>
                            ) : (
                              <>
                                <FileText size={14} className="text-gray-500" />
                              </>
                            )}
                            <span className="truncate">{entry.name}</span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                  <div className="flex-1 flex flex-col p-4 overflow-y-auto bg-gray-50">
                    {loadingFile ? (
                      <div className="flex items-center justify-center h-full">
                        <Loader2 size={32} className="animate-spin text-gray-400" />
                      </div>
                    ) : selectedFile ? (
                      <div className="w-full h-full font-mono text-xs space-y-4">
                        <div className="flex items-center justify-between border-b pb-2">
                          <span className="font-black">{selectedFile.name}</span>
                          <span className="text-gray-400">
                            {selectedFile.mime_type.includes("markdown") ? "Markdown" : selectedFile.mime_type} &bull; {formatFileSize(selectedFile.size)}
                          </span>
                        </div>
                        <pre className="whitespace-pre-wrap break-all text-xs leading-relaxed">
                          {selectedFile.content}
                        </pre>
                      </div>
                    ) : (
                      <div className="flex-1 flex flex-col items-center justify-center text-gray-400">
                        <Folder size={48} strokeWidth={1} className="mb-2 opacity-20" />
                        <span className="text-xs italic">Select a file to view</span>
                      </div>
                    )}
                  </div>
                </div>
              </>
            )}
          </div>
        )}

        {activeTab === 'CHAT' && (
          <div className="h-full flex flex-col">
             <div className="flex-1 overflow-y-auto space-y-6 pb-4">
                {/* Channel mode empty state */}
                {isChannelMode && channelDisplayMessages.length === 0 && !channelIsThinking && !channelIsStreaming && (
                  <div className="h-full flex flex-col justify-center items-center text-gray-400 italic text-sm">
                    <div className="flex items-center gap-2 mb-2">
                      <AtSign size={16} />
                      <span>No messages yet in #{activeChannel!.name}.</span>
                    </div>
                    <span className="text-[10px]">Type @AgentName to mention a specific agent.</span>
                  </div>
                )}
                {/* Agent/Thread mode empty state */}
                {!isChannelMode && displayMessages.length === 0 && !isThinking && !isStreaming && (
                  <div className="h-full flex flex-col justify-center items-center text-gray-400 italic text-sm">
                    {selectedAgent
                      ? `No messages yet. Start a conversation with ${agentName}.`
                      : 'No messages yet. Select an agent or channel to start chatting.'}
                  </div>
                )}

                {(isChannelMode ? channelDisplayMessages : displayMessages).map((msgRaw) => {
                  const msg = msgRaw as (Message & { agentColor?: string; agentEmoji?: string });
                  return (
                  <div key={msg.id} className="flex gap-3 px-2">
                    <div className={cn(
                      "w-8 h-8 brutal-border flex items-center justify-center shrink-0 font-black",
                      msg.sender.isAgent ? (msg.agentColor || "bg-brutal-cyan") : "bg-purple-400"
                    )}>
                      {msg.sender.isAgent ? (
                        <span className="text-sm">{msg.agentEmoji?.charAt(0) || msg.sender.avatar}</span>
                      ) : (
                        <User size={18} />
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">
                          {msg.sender.name}
                        </span>
                        <span className="text-[8px] text-gray-500 uppercase">{msg.timestamp}</span>
                      </div>
                      <div className="text-sm leading-relaxed whitespace-pre-wrap">
                        {/* Highlight @mentions in channel messages */}
                        {isChannelMode
                          ? renderMentionText(msg.content, allAgents)
                          : msg.content
                        }
                      </div>
                      {/* Context info badge for agent messages in channels */}
                      {isChannelMode && msg.sender.isAgent && (
                        <div className="mt-1 flex items-center gap-2 text-[8px] text-gray-400">
                          <span className="px-1 py-0.5 bg-gray-100 brutal-border">SOUL.md</span>
                          <span className="px-1 py-0.5 bg-gray-100 brutal-border">Channel History</span>
                          <span className="px-1 py-0.5 bg-gray-100 brutal-border">MEMORY.md</span>
                        </div>
                      )}
                    </div>
                  </div>
                  );
                })}

                {/* Multi-Agent streaming bubbles */}
                {isChannelMode && channelAgentStreams.map((stream) => {
                  const agentInfo = agentMap.get(stream.agent_id);
                  const cIdx = agentColorMap.get(stream.agent_id) ?? 0;
                  return (
                    <AgentStreamBubble
                      key={stream.agent_id}
                      stream={stream}
                      agentInfo={agentInfo}
                      colorIndex={cIdx}
                    />
                  );
                })}

                {/* Single-agent streaming text display (backward compat, when no agentStreams) */}
                {isChannelMode && channelAgentStreams.length === 0 && channelStreamingText && (
                  <div className="flex gap-3 px-2">
                    <div className="w-8 h-8 brutal-border bg-brutal-cyan flex items-center justify-center shrink-0 font-black">
                      <Bot size={18} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">Agent</span>
                        <span className="text-[8px] text-gray-500 uppercase italic">Streaming...</span>
                      </div>
                      <div className="text-sm leading-relaxed whitespace-pre-wrap">
                        {channelStreamingText}
                        <span className="inline-block w-1.5 h-4 bg-brutal-cyan ml-0.5 animate-pulse" />
                      </div>
                    </div>
                  </div>
                )}

                {/* Streaming text display (thread mode) */}
                {!isChannelMode && streamingText && (
                  <div className="flex gap-3 px-2">
                    <div className="w-8 h-8 brutal-border bg-brutal-cyan flex items-center justify-center shrink-0 font-black">
                      <Bot size={18} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">{agentName}</span>
                        <span className="text-[8px] text-gray-500 uppercase italic">Streaming...</span>
                      </div>
                      <div className="text-sm leading-relaxed whitespace-pre-wrap">
                        {streamingText}
                        <span className="inline-block w-1.5 h-4 bg-brutal-cyan ml-0.5 animate-pulse" />
                      </div>
                    </div>
                  </div>
                )}

                {/* Thinking indicator (thread mode) */}
                {!isChannelMode && isThinking && !streamingText && (
                  <div className="flex gap-3 px-2 animate-pulse">
                    <div className="w-8 h-8 brutal-border bg-brutal-cyan flex items-center justify-center shrink-0">
                      <Bot size={18} />
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">{agentName}</span>
                        <span className="text-[8px] text-gray-500 uppercase italic">Thinking...</span>
                      </div>
                      <div className="h-4 bg-gray-200 w-2/3 brutal-border-b" />
                    </div>
                  </div>
                )}

                {/* Thinking indicator (channel mode, single agent) */}
                {isChannelMode && channelIsThinking && !channelStreamingText && channelAgentStreams.length === 0 && (
                  <div className="flex gap-3 px-2 animate-pulse">
                    <div className="w-8 h-8 brutal-border bg-brutal-cyan flex items-center justify-center shrink-0">
                      <Bot size={18} />
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">Agent</span>
                        <span className="text-[8px] text-gray-500 uppercase italic">Thinking...</span>
                      </div>
                      <div className="h-4 bg-gray-200 w-2/3 brutal-border-b" />
                    </div>
                  </div>
                )}
                <div ref={messagesEndRef} />
             </div>

             <div className="mt-4">
                {isChannelMode ? (
                  /* @Mention autocomplete input for channel mode */
                  <MentionAutocomplete
                    value={inputValue}
                    onChange={setInputValue}
                    members={channelMembers}
                    disabled={!activeChannel || channelIsStreaming || channelIsThinking}
                    placeholder={`Message #${activeChannel!.name}... (type @ to mention)`}
                    onSend={handleSendChannelMessage}
                  />
                ) : (
                  /* Standard textarea for thread mode */
                  <div className="relative">
                    <textarea
                      value={inputValue}
                      onChange={(e) => setInputValue(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && !e.shiftKey) {
                          e.preventDefault();
                          handleSendMessage();
                        }
                      }}
                      placeholder={
                        selectedAgent
                          ? `Message @${agentName}...`
                          : 'Select an agent or channel to start chatting...'
                      }
                      disabled={!selectedAgent || isStreaming || isThinking}
                      className={cn(
                        "w-full brutal-border bg-white p-3 min-h-[100px] text-sm focus:outline-none focus:bg-brutal-bg resize-none",
                        (!selectedAgent || isStreaming || isThinking) && "opacity-50 cursor-not-allowed"
                      )}
                    />
                  </div>
                )}
                <div className="flex items-center justify-between mt-2">
                  <div className="flex gap-1">
                    <button className="p-2 brutal-border hover:bg-gray-100">
                      <Plus size={20} />
                    </button>
                    <button className="p-2 brutal-border hover:bg-gray-100">
                      <ImageIcon size={20} />
                    </button>
                  </div>
                  <div className="flex items-center gap-4">
                    <label className="flex items-center gap-2 text-xs font-bold cursor-pointer">
                      <input type="checkbox" className="brutal-border w-4 h-4 accent-black" />
                      As Task
                    </label>
                    <button
                      onClick={isChannelMode ? handleSendChannelMessage : handleSendMessage}
                      disabled={!canSend}
                      className={cn(
                        "brutal-btn flex items-center gap-2",
                        !canSend ? "bg-gray-200 text-gray-400 cursor-not-allowed" : "bg-brutal-pink text-white"
                      )}
                    >
                      Send <Send size={16} />
                    </button>
                  </div>
                </div>
             </div>
          </div>
        )}

        {activeTab === 'SKILLS' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {[
                { name: 'Terminal Access', icon: Terminal, desc: 'Execute commands in the workspace environment.', status: 'Active', color: 'bg-brutal-green' },
                { name: 'Web Search', icon: Globe, desc: 'Browse the web for real-time information.', status: 'Active', color: 'bg-brutal-cyan' },
                { name: 'Code Analysis', icon: Code, desc: 'Deep understanding of codebase and dependencies.', status: 'Active', color: 'bg-brutal-pink' },
                { name: 'Database Query', icon: Database, desc: 'Interact with connected databases.', status: 'Inactive', color: 'bg-gray-300' },
                { name: 'Knowledge Base', icon: Search, desc: 'Search through internal documentation.', status: 'Active', color: 'bg-brutal-yellow' },
              ].map((skill, i) => (
                <div key={i} className="brutal-card group hover:translate-x-[-2px] hover:translate-y-[-2px] transition-all">
                  <div className="flex items-start justify-between mb-3">
                    <div className={cn("p-2 brutal-border", skill.color)}>
                      <skill.icon size={24} />
                    </div>
                    <span className={cn(
                      "text-[8px] font-black px-1.5 py-0.5 brutal-border uppercase",
                      skill.status === 'Active' ? "bg-brutal-green" : "bg-gray-200"
                    )}>
                      {skill.status}
                    </span>
                  </div>
                  <h4 className="font-black text-sm mb-1">{skill.name}</h4>
                  <p className="text-xs text-gray-600 leading-tight">{skill.desc}</p>
                  <div className="mt-4 flex justify-end">
                    <button className="text-[10px] font-black uppercase underline hover:text-brutal-pink">Configure</button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'ACTIVITY' && (
          <div className="space-y-1 font-mono text-[11px]">
            {logs.map((log, i) => (
              <div key={i} className="flex gap-4 py-1 hover:bg-white/50 px-2 transition-colors">
                <span className="text-gray-400 shrink-0">{log.time}</span>
                <div className="flex items-center gap-2 shrink-0 w-24">
                  <Circle
                    size={8}
                    fill={
                      log.status === 'Thinking' ? '#FFDE00' :
                      log.status === 'Idle' ? '#39FF14' :
                      log.status === 'Working' ? '#00E5FF' :
                      log.status === 'Error' ? '#FF4081' : '#000000'
                    }
                  />
                  <span className={cn("font-bold", log.color)}>{log.status}</span>
                </div>
                <span className="text-gray-600 italic truncate">{log.text}</span>
              </div>
            ))}
          </div>
        )}

        {activeTab === 'PROFILE' && (
          <div className="space-y-6">
            {!selectedAgent ? (
              // No agent selected - empty state
              <div className="flex flex-col items-center justify-center py-16 brutal-border bg-white brutal-shadow-sm">
                <Bot size={48} className="text-gray-300 mb-4" />
                <p className="text-gray-500 text-sm">Select an agent to view profile</p>
              </div>
            ) : profileLoading ? (
              // Loading state
              <div className="flex flex-col items-center justify-center py-16 brutal-border bg-white brutal-shadow-sm">
                <div className="w-8 h-8 border-4 border-brutal-cyan border-t-transparent rounded-full animate-spin mb-4" />
                <p className="text-gray-500 text-sm">Loading profile...</p>
              </div>
            ) : profileData.identity ? (
              // Profile content
              <>
                {/* Agent Header */}
                <div className="flex flex-col items-center py-8 brutal-border bg-white brutal-shadow-sm">
                  <div className="w-20 h-20 brutal-border bg-brutal-cyan flex items-center justify-center mb-4">
                    {profileData.identity.emoji ? (
                      <span className="text-4xl">{profileData.identity.emoji}</span>
                    ) : (
                      <Bot size={48} />
                    )}
                  </div>
                  <h2 className="text-2xl font-black">{profileData.identity.name}</h2>
                  <span className="text-gray-500 font-mono text-xs">@{profileData.identity.agent_id}</span>
                  {profileData.identity.creature && (
                    <span className="mt-1 text-xs text-gray-400">
                      {profileData.identity.creature} · {profileData.identity.vibe}
                    </span>
                  )}
                </div>

                {/* Role Section */}
                <section className="space-y-2">
                  <div className="flex items-center justify-between">
                    <h3 className="font-black text-xs uppercase tracking-widest text-gray-500">Role</h3>
                    <button className="p-1 hover:bg-gray-200 rounded"><Plus size={14}/></button>
                  </div>
                  <div className="brutal-card text-sm leading-relaxed">
                    {profileData.context?.system_prompt || 'No role description set'}
                  </div>
                </section>

                {/* Configuration Section */}
                <section className="space-y-2">
                  <h3 className="font-black text-xs uppercase tracking-widest text-gray-500">Configuration</h3>
                  <div className="brutal-card grid grid-cols-2 gap-6">
                    <div>
                      <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Runtime</label>
                      <div className="brutal-btn bg-brutal-cyan text-xs inline-block">
                        {selectedAgent.runtime_status === 'available' ? 'Claude Code' : 'Not Available'}
                      </div>
                    </div>
                    <div>
                      <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Status</label>
                      <div className="flex items-center gap-1">
                        <Circle
                          size={8}
                          fill={getRuntimeStatusColor(selectedAgent.runtime_status)}
                          className="shrink-0"
                        />
                        <span className="text-xs font-bold">
                          {getRuntimeStatusLabel(selectedAgent.runtime_status)}
                        </span>
                      </div>
                    </div>
                    {selectedAgent.runtime_version && (
                      <div>
                        <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Version</label>
                        <div className="text-xs font-mono font-bold">v{selectedAgent.runtime_version}</div>
                      </div>
                    )}
                    {profileData.workspace && (
                      <div>
                        <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Workspace</label>
                        <div className="text-xs font-mono font-bold truncate max-w-[200px]" title={profileData.workspace.workspace_root}>
                          {profileData.workspace.workspace_root.split('/').pop()}
                        </div>
                      </div>
                    )}
                  </div>
                </section>
              </>
            ) : (
              // Error state - no profile data
              <div className="flex flex-col items-center justify-center py-16 brutal-border bg-white brutal-shadow-sm">
                <p className="text-gray-500 text-sm">Failed to load profile data</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
