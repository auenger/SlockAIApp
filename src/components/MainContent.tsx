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
  ChevronDown,
  ChevronUp,
  Bot,
  Circle,
  Square,
  RotateCcw,
  Copy,
  Send,
  Image as ImageIcon,
  Terminal,
  Code,
  Globe,
  Hash,
  AtSign,
  FileText,
  Loader2,
  Settings,
  AlertCircle,
  Filter,
  Pencil,
  FolderOpen,
  Wrench,
} from 'lucide-react';
import { cn } from '../lib/utils';
import { TabType, Task, Message, AgentWithRuntime, Channel, ChannelMessage, ContentBlock } from '../types';
import { getRuntimeStatusColor, getRuntimeStatusLabel } from '../lib/useAgentStatus';
import { useAgentProfile } from '../lib/useAgentProfile';
import { useThreadChat } from '../lib/useThreadChat';
import { useWorkspace } from '../lib/useWorkspace';
import { openWorkspaceInFinder } from '../lib/ipc';
import { useSkills } from '../lib/useSkills';
import { SkillFormModal } from './SkillsPanel';
import type { SkillInfo } from '../types';
import { useActivityLog, getActivityTypeConfig } from '../lib/useActivityLog';
import { MentionAutocomplete, renderMentionText } from './MentionAutocomplete';
import { useUserProfile } from '../lib/useUserProfile';
import { AgentIcon } from './AgentIcon';
import { EditAgentModal } from './EditAgentModal';
import { MarkdownRenderer } from './markdown';
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
// Tool icon mapping
// ---------------------------------------------------------------------------

const TOOL_ICONS: Record<string, React.ElementType> = {
  Read: FileText,
  Write: Pencil,
  Bash: Terminal,
  Edit: Code,
  Glob: Folder,
  Grep: Filter,
};

function getToolIcon(name: string): React.ElementType {
  return TOOL_ICONS[name] || Wrench;
}

// ---------------------------------------------------------------------------
// ContentBlockCard — renders a single tool_use / tool_result block
// ---------------------------------------------------------------------------

interface ContentBlockCardProps {
  block: ContentBlock;
}

/** Truncate a string to a max length with ellipsis */
function truncate(str: string, maxLen: number): string {
  if (str.length <= maxLen) return str;
  return str.slice(0, maxLen) + '...';
}

/** Format the input params for a tool_use preview */
function formatInputPreview(input?: Record<string, unknown>): string {
  if (!input) return '';
  // Show the most relevant param depending on tool
  if (input.file_path) return String(input.file_path);
  if (input.command) return truncate(String(input.command), 60);
  if (input.pattern) return String(input.pattern);
  // Fallback: first value
  const vals = Object.values(input);
  if (vals.length > 0) return truncate(String(vals[0]), 60);
  return '';
}

/** Format tool_result content for preview */
function formatResultPreview(content?: string | unknown[]): string {
  if (!content) return '';
  if (typeof content === 'string') return truncate(content, 80);
  // Array of content items — extract text
  if (Array.isArray(content)) {
    const textParts = content
      .filter((c): c is Record<string, unknown> => typeof c === 'object' && c !== null)
      .filter((c) => c.type === 'text')
      .map((c) => String(c.text ?? ''));
    if (textParts.length > 0) return truncate(textParts.join(' '), 80);
  }
  return '';
}

const ContentBlockCard: React.FC<ContentBlockCardProps> = ({ block }) => {
  const [expanded, setExpanded] = useState(false);

  if (block.type === 'tool_use') {
    const Icon = getToolIcon(block.name || '');
    const preview = formatInputPreview(block.input);
    const detail = block.input ? JSON.stringify(block.input, null, 2) : '';

    return (
      <div className="brutal-border bg-gray-50 my-1 text-xs">
        <button
          onClick={() => setExpanded(!expanded)}
          className="w-full flex items-center gap-1.5 px-2 py-1.5 hover:bg-gray-100 transition-colors text-left"
        >
          {expanded ? <ChevronUp size={10} className="shrink-0" /> : <ChevronDown size={10} className="shrink-0" />}
          <Icon size={11} className="shrink-0 text-brutal-cyan" />
          <span className="font-black uppercase text-[10px] px-1 bg-brutal-cyan text-white">
            {block.name || 'tool'}
          </span>
          {preview && (
            <span className="text-gray-600 font-mono truncate text-[10px]">{preview}</span>
          )}
        </button>
        {expanded && detail && (
          <pre className="px-2 pb-2 text-[10px] font-mono text-gray-700 whitespace-pre-wrap break-all overflow-hidden max-h-48 overflow-y-auto">
            {detail}
          </pre>
        )}
      </div>
    );
  }

  // tool_result
  const preview = formatResultPreview(block.content);
  const detail = typeof block.content === 'string'
    ? block.content
    : JSON.stringify(block.content, null, 2);

  return (
    <div className="brutal-border bg-gray-50/50 my-1 text-xs">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-1.5 px-2 py-1.5 hover:bg-gray-100 transition-colors text-left"
      >
        {expanded ? <ChevronUp size={10} className="shrink-0" /> : <ChevronDown size={10} className="shrink-0" />}
        <span className="font-black uppercase text-[10px] px-1 bg-brutal-green text-white">
          result
        </span>
        {preview && (
          <span className="text-gray-500 font-mono truncate text-[10px]">{preview}</span>
        )}
      </button>
      {expanded && detail && (
        <pre className="px-2 pb-2 text-[10px] font-mono text-gray-600 whitespace-pre-wrap break-all overflow-hidden max-h-48 overflow-y-auto">
          {detail}
        </pre>
      )}
    </div>
  );
};

// ---------------------------------------------------------------------------
// Agent Streaming Bubble
// ---------------------------------------------------------------------------

interface AgentStreamBubbleProps {
  stream: AgentStreamState;
  agentInfo?: AgentWithRuntime;
  colorIndex: number;
  allAgents: AgentWithRuntime[];
  agentColorMap: Map<string, number>;
  userName?: string;
}

const AgentStreamBubble: React.FC<AgentStreamBubbleProps> = ({
  stream,
  agentInfo,
  colorIndex,
  allAgents,
  agentColorMap,
  userName,
}) => {
  const agentName = agentInfo?.agent.name || stream.agent_id;
  const bgColor = getAgentColor(colorIndex);
  const hasContentBlocks = stream.contentBlocks && stream.contentBlocks.length > 0;

  return (
    <div className="flex gap-3 px-2">
      <AgentIcon
        icon={agentInfo?.agent.icon}
        emoji={agentInfo?.agent.emoji}
        size="md"
        bgColor={bgColor}
      />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className={cn("font-black text-xs", bgColor === 'bg-brutal-yellow' ? 'text-black' : 'text-black')}>
            {agentName}
          </span>
          {stream.thinking ? (
            <span className="text-[8px] text-gray-500 uppercase italic flex items-center gap-1">
              Thinking
              <span className="inline-flex gap-[2px]">
                <span className="w-[3px] h-[3px] bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                <span className="w-[3px] h-[3px] bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                <span className="w-[3px] h-[3px] bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
              </span>
            </span>
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
        {stream.thinking && !stream.text && !hasContentBlocks ? (
          <div>
            {stream.statusMessage && (
              <div className="text-[10px] text-gray-500 font-mono italic mb-1">
                {stream.statusMessage}
              </div>
            )}
          </div>
        ) : (
          <>
            <div className="text-sm leading-relaxed">
              <MarkdownRenderer content={stream.text || ''} allAgents={allAgents} agentColorMap={agentColorMap} userName={userName} />
              {stream.streaming && (
                <span className="inline-flex gap-[2px] ml-1">
                  <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                  <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                  <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
                </span>
              )}
            </div>
            {/* Status message during thinking with tool activity */}
            {stream.thinking && stream.statusMessage && !stream.text && (
              <div className="text-[10px] text-gray-500 font-mono italic mb-1">
                {stream.statusMessage}
              </div>
            )}
            {/* Tool call cards — shown during streaming */}
            {hasContentBlocks && (
              <div className="mt-2 space-y-1">
                {stream.contentBlocks.slice(-10).map((block, idx) => (
                  <ContentBlockCard key={`${block.type}-${block.id || block.tool_use_id || idx}`} block={block} />
                ))}
              </div>
            )}
          </>
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
  onSendChannelMessage?: (channelId: string, message: string, userName?: string) => Promise<void>;
  /** Whether a channel message is streaming */
  channelIsStreaming?: boolean;
  /** Whether the channel agent is thinking */
  channelIsThinking?: boolean;
  /** Buffered streaming text for channel */
  channelStreamingText?: string;
  /** Per-agent streaming states for multi-Agent responses */
  channelAgentStreams?: AgentStreamState[];
  /** Delete the currently active channel */
  onDeleteChannel?: (channelId: string) => void;
  /** Delete an agent by ID */
  onDeleteAgent?: (agentId: string) => void;
  /** Refresh the current view data (channel or thread) */
  onRefresh?: () => void;
  /** Stop the currently running agent session */
  onStopSession?: () => void;
}

export const MainContent: React.FC<MainContentProps> = ({
  activeTab,
  onTabChange,
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
  onDeleteChannel,
  onDeleteAgent,
  onRefresh,
  onStopSession,
}) => {
  const [taskFilter, setTaskFilter] = useState('All');
  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<{ type: 'channel' | 'agent'; id: string; name: string } | null>(null);

  // User profile for self-mention
  const { profile: userProfile } = useUserProfile();

  // Mention toast state: 'self' = user mentioned themselves, 'agent' = agent mentioned user
  const [mentionToast, setMentionToast] = useState<'self' | 'agent' | null>(null);

  // Detect when an Agent @mentions the user in incoming messages
  // Shows toast + system notification
  const lastCheckedMsgCountRef = useRef(0);
  useEffect(() => {
    if (!activeChannel || !activeChannel.messages) return;
    const userName = userProfile.name;
    if (!userName || userName === 'User') return;

    const messages = activeChannel.messages;
    // Only check new messages since last check
    if (messages.length <= lastCheckedMsgCountRef.current) {
      lastCheckedMsgCountRef.current = messages.length;
      return;
    }

    const escaped = userName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const mentionRegex = new RegExp(`@${escaped}\\b`, 'i');

    let foundUserMention = false;
    for (let i = lastCheckedMsgCountRef.current; i < messages.length; i++) {
      const msg = messages[i];
      // Only check agent messages (not user's own messages)
      if (msg.sender_type === 'agent' && mentionRegex.test(msg.content)) {
        foundUserMention = true;
        break;
      }
    }

    lastCheckedMsgCountRef.current = messages.length;

    if (foundUserMention) {
      setMentionToast('agent');
      setTimeout(() => setMentionToast(null), 4000);

      // System-level notification
      try {
        if ('Notification' in window) {
          const showNotif = () => {
            new Notification('You were mentioned', {
              body: `An agent mentioned you (@${userName}) in #${activeChannel.name}.`,
              silent: false,
            });
          };
          if (Notification.permission === 'granted') {
            showNotif();
          } else if (Notification.permission !== 'denied') {
            Notification.requestPermission().then((perm) => {
              if (perm === 'granted') showNotif();
            });
          }
        }
      } catch {
        // Notification API not available
      }
    }
  }, [activeChannel?.messages, userProfile.name, activeChannel?.name]);

  // Skills management state
  const {
    skills,
    loading: skillsLoading,
    error: skillsError,
    loadSkills,
    add: addSkillAction,
    update: updateSkillAction,
    remove: removeSkillAction,
    clearError: clearSkillsError,
  } = useSkills();
  const [skillModalOpen, setSkillModalOpen] = useState(false);
  const [editingSkill, setEditingSkill] = useState<SkillInfo | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [skillSubmitting, setSkillSubmitting] = useState(false);

  // Edit agent modal state
  const [showEditAgent, setShowEditAgent] = useState(false);

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

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [activeThread?.messages, activeChannel?.messages, isThinking, streamingText, channelIsThinking, channelStreamingText, channelAgentStreams]);

  // When activeThreadId changes (from sidebar), load the thread
  // The thread selection is now handled by App.tsx which also sets the agent
  // We only need to react to changes in selectedAgent for profile loading etc.

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

  // Load skills when selectedAgent changes or tab becomes SKILLS
  useEffect(() => {
    if (selectedAgent?.agent.agent_id && activeTab === 'SKILLS') {
      loadSkills(selectedAgent.agent.agent_id);
    }
  }, [selectedAgent?.agent.agent_id, activeTab, loadSkills]);

  // Activity log hook (filtered by selected agent if any)
  const [activityAgentFilter, setActivityAgentFilter] = useState<string | null>(null);
  const {
    entries: activityEntries,
    total: activityTotal,
    loading: activityLoading,
    refresh: refreshActivity,
    loadMore: loadMoreActivity,
    hasMore: hasMoreActivity,
  } = useActivityLog(activityAgentFilter);

  const handleSendMessage = async () => {
    if (!inputValue.trim() || !selectedAgent) return;
    if (isStreaming || isThinking) return;

    const agentId = selectedAgent.agent.agent_id;
    let threadId = activeThread?.id;

    // If no active thread, create one
    if (!threadId) {
      try {
        const newThread = await createNewThread(agentId, selectedAgent.agent.name);
        threadId = newThread.id;
        onThreadCreated?.(threadId);
      } catch (err) {
        return;
      }
    }

    const userInput = inputValue;
    setInputValue('');

    try {
      await send(agentId, threadId, userInput);
    } catch (error) {
      console.error("Agent Error:", error);
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
  const channelDisplayMessages: (Message & { agentColor?: string; agentEmoji?: string; contentBlocks?: ContentBlock[] })[] = (() => {
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
        contentBlocks: msg.content_blocks,
      };
    });
  })();

  /** Handle sending a message in channel mode */
  const handleSendChannelMessage = async () => {
    if (!inputValue.trim() || !activeChannel || !onSendChannelMessage) return;
    if (channelIsStreaming || channelIsThinking) return;

    const userInput = inputValue;
    setInputValue('');

    // Check for self-mention and show reminder
    const userName = userProfile.name;
    if (userName) {
      const escaped = userName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      // Match @UserName format
      const hasSelfMention = new RegExp(`@${escaped}\\b`, 'i').test(userInput);
      if (hasSelfMention) {
        setMentionToast('self');
        setTimeout(() => setMentionToast(null), 4000);

        // System-level notification
        try {
          if ('Notification' in window) {
            const showNotif = () => {
              new Notification('Self-mention', {
                body: `You mentioned yourself (@${userName}) — agents can see this.`,
                silent: true,
              });
            };
            if (Notification.permission === 'granted') {
              showNotif();
            } else if (Notification.permission !== 'denied') {
              Notification.requestPermission().then((perm) => {
                if (perm === 'granted') showNotif();
              });
            }
          }
        } catch {
          // Notification API not available
        }
      }
    }

    try {
      await onSendChannelMessage(
        activeChannel.id,
        userInput,
        userProfile.name !== 'User' ? userProfile.name : undefined,
      );
    } catch (error) {
      console.error("Channel Error:", error);
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

  const tasks: Task[] = [];

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
          {isChannelMode ? (
            <div className="w-10 h-10 brutal-border bg-brutal-cyan flex items-center justify-center">
              <Hash size={24} />
            </div>
          ) : selectedAgent ? (
            <AgentIcon
              icon={selectedAgent.agent.icon}
              emoji={selectedAgent.agent.emoji}
              size="lg"
              bgColor="bg-brutal-cyan"
            />
          ) : (
            <div className="w-10 h-10 brutal-border bg-brutal-cyan flex items-center justify-center">
              <Bot size={24} />
            </div>
          )}
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
                    <AgentIcon
                      key={member.agent_id}
                      icon={memberAgent?.agent.icon}
                      emoji={memberAgent?.agent.emoji}
                      size="sm"
                      bgColor="bg-brutal-cyan"
                      title={memberAgent?.agent.name || member.agent_id}
                    />
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
          {/* Stop/Pause button: stop the current agent session */}
          <button
            onClick={onStopSession}
            disabled={!channelIsStreaming && !channelIsThinking && !isStreaming && !isThinking}
            className={cn(
              "p-2 brutal-border",
              (channelIsStreaming || channelIsThinking || isStreaming || isThinking)
                ? "hover:bg-red-50 bg-brutal-pink text-white"
                : "opacity-40 cursor-not-allowed"
            )}
            title="Stop current session"
          >
            <Square size={18} />
          </button>
          {/* Refresh button: reload current view data */}
          <button
            onClick={async () => {
              if (!onRefresh || refreshing) return;
              setRefreshing(true);
              try {
                await onRefresh();
                // Re-select thread in thread mode to reload messages
                if (!isChannelMode && selectedAgent && activeThreadId) {
                  selectThread(selectedAgent.agent.agent_id, activeThreadId);
                }
              } finally {
                // Brief animation delay
                setTimeout(() => setRefreshing(false), 600);
              }
            }}
            disabled={!onRefresh}
            className={cn(
              "p-2 brutal-border hover:bg-gray-100",
              !onRefresh && "opacity-40 cursor-not-allowed"
            )}
            title="Refresh current view"
          >
            <RotateCcw size={18} className={refreshing ? "animate-spin" : ""} />
          </button>
          {/* Delete button: delete channel or agent */}
          <button
            onClick={() => {
              if (isChannelMode && activeChannel) {
                setDeleteConfirm({ type: 'channel', id: activeChannel.id, name: activeChannel.name });
              } else if (selectedAgent) {
                setDeleteConfirm({ type: 'agent', id: selectedAgent.agent.agent_id, name: selectedAgent.agent.name });
              }
            }}
            disabled={!activeChannel && !selectedAgent}
            className={cn(
              "p-2 brutal-border bg-brutal-yellow hover:bg-yellow-400",
              !activeChannel && !selectedAgent && "opacity-40 cursor-not-allowed"
            )}
            title="Delete"
          >
            <Trash2 size={18} />
          </button>
        </div>
      </div>

      {/* Delete Confirmation Dialog (matches Sidebar pattern) */}
      {deleteConfirm && (
        <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50" onClick={() => setDeleteConfirm(null)}>
          <div className="bg-white brutal-border brutal-shadow p-4 w-72 space-y-3" onClick={(e) => e.stopPropagation()}>
            <div className="font-black text-sm uppercase">Confirm Delete</div>
            <div className="text-xs text-gray-600">
              Are you sure you want to delete <span className="font-bold">{deleteConfirm.name}</span>?
              This action cannot be undone.
            </div>
            <div className="flex items-center justify-end gap-2">
              <button
                onClick={() => setDeleteConfirm(null)}
                className="px-3 py-1 brutal-border bg-gray-200 text-xs font-black hover:bg-gray-300"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  if (!deleteConfirm) return;
                  if (deleteConfirm.type === 'channel') onDeleteChannel?.(deleteConfirm.id);
                  else if (deleteConfirm.type === 'agent') onDeleteAgent?.(deleteConfirm.id);
                  setDeleteConfirm(null);
                }}
                className="px-3 py-1 brutal-border bg-brutal-pink text-white text-xs font-black hover:bg-red-500"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}

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
                {['All', 'Todo', 'In Progress', 'In Review', 'Done'].map(f => (
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
                      task.status === 'todo' ? "bg-brutal-yellow" : "bg-brutal-cyan"
                    )}>
                      {task.status}
                    </span>
                    <span className="font-bold text-sm">{task.title}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {task.assigneeId && (
                      <div className="flex items-center gap-1 px-1.5 py-0.5 brutal-border bg-gray-50 text-[10px] font-bold">
                        <span className="text-gray-500">@</span>{task.assigneeId}
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
                  <button
                    onClick={() => openWorkspaceInFinder(selectedAgent.agent.agent_id).catch(err => console.error('[Workspace] Failed to open in Finder:', err))}
                    className="p-0.5 brutal-border hover:bg-gray-100"
                    title="Open in Finder"
                  >
                    <FolderOpen size={10} />
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
                  const msg = msgRaw as (Message & { agentColor?: string; agentEmoji?: string; contentBlocks?: ContentBlock[] });
                  const hasBlocks = msg.contentBlocks && msg.contentBlocks.length > 0;
                  return (
                  <div key={msg.id} className="flex gap-3 px-2">
                    {msg.sender.isAgent ? (
                      <AgentIcon
                        icon={allAgents.find(a => a.agent.name === msg.sender.name)?.agent.icon}
                        emoji={msg.agentEmoji || msg.sender.avatar}
                        size="md"
                        bgColor={msg.agentColor || "bg-brutal-cyan"}
                      />
                    ) : (
                      <div className="w-8 h-8 brutal-border bg-purple-400 flex items-center justify-center shrink-0">
                        <User size={18} className="text-white" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">
                          {msg.sender.name}
                        </span>
                        <span className="text-[8px] text-gray-500 uppercase">{msg.timestamp}</span>
                      </div>
                      <div className="text-sm leading-relaxed">
                        {/* Render with MarkdownRenderer for agent messages, plain text for user */}
                        {msg.sender.isAgent ? (
                          <MarkdownRenderer content={msg.content} allAgents={allAgents} agentColorMap={agentColorMap} userName={userProfile.name !== 'User' ? userProfile.name : undefined} />
                        ) : isChannelMode ? (
                          <span className="whitespace-pre-wrap">{renderMentionText(msg.content, allAgents, undefined, agentColorMap, userProfile.name !== 'User' ? userProfile.name : undefined)}</span>
                        ) : (
                          <span className="whitespace-pre-wrap">{msg.content}</span>
                        )}
                      </div>
                      {/* Persisted content_blocks for historical agent messages */}
                      {isChannelMode && msg.sender.isAgent && hasBlocks && (
                        <div className="mt-2 space-y-1">
                          {msg.contentBlocks!.map((block, idx) => (
                            <ContentBlockCard key={`hist-${msg.id}-${block.type}-${block.id || block.tool_use_id || idx}`} block={block} />
                          ))}
                        </div>
                      )}
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
                      allAgents={allAgents}
                      agentColorMap={agentColorMap}
                      userName={userProfile.name !== 'User' ? userProfile.name : undefined}
                    />
                  );
                })}

                {/* Single-agent streaming text display (backward compat, when no agentStreams) */}
                {isChannelMode && channelAgentStreams.length === 0 && channelStreamingText && (
                  <div className="flex gap-3 px-2">
                    <AgentIcon
                      icon={null}
                      emoji="B"
                      size="md"
                      bgColor="bg-brutal-cyan"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">Agent</span>
                        <span className="text-[8px] text-gray-500 uppercase italic">Streaming...</span>
                      </div>
                      <div className="text-sm leading-relaxed">
                        <MarkdownRenderer content={channelStreamingText || ''} allAgents={allAgents} agentColorMap={agentColorMap} userName={userProfile.name !== 'User' ? userProfile.name : undefined} />
                        <span className="inline-flex gap-[2px] ml-1">
                          <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                          <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                          <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
                        </span>
                      </div>
                    </div>
                  </div>
                )}

                {/* Streaming text display (thread mode) */}
                {!isChannelMode && streamingText && (
                  <div className="flex gap-3 px-2">
                    <AgentIcon
                      icon={selectedAgent?.agent.icon}
                      emoji={selectedAgent?.agent.emoji}
                      size="md"
                      bgColor="bg-brutal-cyan"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">{agentName}</span>
                        <span className="text-[8px] text-gray-500 uppercase italic">Streaming...</span>
                      </div>
                      <div className="text-sm leading-relaxed">
                        <MarkdownRenderer content={streamingText || ''} />
                        <span className="inline-flex gap-[2px] ml-1">
                          <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                          <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                          <span className="w-[3px] h-[3px] bg-brutal-cyan rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
                        </span>
                      </div>
                    </div>
                  </div>
                )}

                {/* Thinking indicator (thread mode) */}
                {!isChannelMode && isThinking && !streamingText && (
                  <div className="flex gap-3 px-2 animate-pulse">
                    <AgentIcon
                      icon={selectedAgent?.agent.icon}
                      emoji={selectedAgent?.agent.emoji}
                      size="md"
                      bgColor="bg-brutal-cyan"
                    />
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
                    <AgentIcon
                      icon={null}
                      emoji="B"
                      size="md"
                      bgColor="bg-brutal-cyan"
                    />
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
                  <>
                  {/* @Mention autocomplete input for channel mode */}
                  <MentionAutocomplete
                    value={inputValue}
                    onChange={setInputValue}
                    members={channelMembers}
                    disabled={!activeChannel || channelIsStreaming || channelIsThinking}
                    placeholder={`Message #${activeChannel!.name}... (type @ to mention)`}
                    onSend={handleSendChannelMessage}
                    userName={userProfile.name !== 'User' ? userProfile.name : undefined}
                  />

                  </>
                ) : (
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
            {/* Error display */}
            {skillsError && (
              <div className="flex items-center gap-2 p-3 brutal-border bg-red-50 text-red-600 text-xs font-bold">
                <AlertCircle size={14} />
                {skillsError}
                <button onClick={clearSkillsError} className="ml-auto underline">Dismiss</button>
              </div>
            )}

            {/* No agent selected */}
            {!selectedAgent ? (
              <div className="flex flex-col items-center justify-center py-16 brutal-border bg-white">
                <Zap size={48} className="text-gray-300 mb-4" />
                <p className="text-gray-500 text-sm">Select an agent to manage skills</p>
              </div>
            ) : (
              <>
                {/* Header with add button */}
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="font-black text-sm uppercase tracking-widest text-gray-500">
                      Skills ({skills.length})
                    </h3>
                  </div>
                  <button
                    onClick={() => {
                      setEditingSkill(null);
                      setSkillModalOpen(true);
                    }}
                    className="brutal-btn bg-brutal-pink text-white text-xs flex items-center gap-1"
                  >
                    <Plus size={14} /> Add Skill
                  </button>
                </div>

                {/* Loading */}
                {skillsLoading ? (
                  <div className="flex items-center justify-center py-12">
                    <Loader2 size={32} className="animate-spin text-gray-400" />
                  </div>
                ) : skills.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-12 brutal-border bg-white">
                    <Zap size={48} strokeWidth={1} className="text-gray-300 mb-4" />
                    <p className="text-gray-500 text-sm italic">No skills configured</p>
                    <p className="text-gray-400 text-[10px] mt-1">Click "Add Skill" to get started</p>
                  </div>
                ) : (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {skills.map((skill) => {
                      // Icon mapping by skill type
                      const typeIcon = skill.skill_type === 'MCP Server' ? Globe
                        : skill.skill_type === 'Tool' ? Terminal
                        : Code;
                      const typeColor = skill.skill_type === 'MCP Server' ? 'bg-brutal-cyan'
                        : skill.skill_type === 'Tool' ? 'bg-brutal-green'
                        : 'bg-brutal-pink';
                      const statusColor = skill.status === 'Active' ? 'bg-brutal-green'
                        : skill.status === 'Connecting' ? 'bg-brutal-yellow'
                        : skill.status === 'Error' ? 'bg-red-200'
                        : 'bg-gray-200';

                      return (
                        <div key={skill.id} className="brutal-card group hover:translate-x-[-2px] hover:translate-y-[-2px] transition-all">
                          <div className="flex items-start justify-between mb-3">
                            <div className={cn("p-2 brutal-border", typeColor)}>
                              {React.createElement(typeIcon, { size: 24 })}
                            </div>
                            <span className={cn(
                              "text-[8px] font-black px-1.5 py-0.5 brutal-border uppercase",
                              statusColor
                            )}>
                              {skill.status}
                            </span>
                          </div>
                          <h4 className="font-black text-sm mb-1">{skill.name}</h4>
                          <span className="text-[10px] text-gray-400 font-mono">{skill.skill_type}</span>
                          <div className="mt-3 flex items-center justify-between">
                            <span className="text-[8px] text-gray-400">
                              {new Date(skill.updated_at).toLocaleDateString()}
                            </span>
                            <div className="flex items-center gap-1">
                              <button
                                onClick={() => {
                                  setEditingSkill(skill);
                                  setSkillModalOpen(true);
                                }}
                                className="p-1 brutal-border hover:bg-gray-100 text-gray-500"
                                title="Configure"
                              >
                                <Settings size={12} />
                              </button>
                              {deleteConfirmId === skill.id ? (
                                <div className="flex items-center gap-1">
                                  <button
                                    onClick={async () => {
                                      setSkillSubmitting(true);
                                      try {
                                        await removeSkillAction(selectedAgent.agent.agent_id, skill.id);
                                        setDeleteConfirmId(null);
                                      } finally {
                                        setSkillSubmitting(false);
                                      }
                                    }}
                                    disabled={skillSubmitting}
                                    className="px-1.5 py-0.5 brutal-border bg-brutal-pink text-white text-[8px] font-black"
                                  >
                                    Confirm
                                  </button>
                                  <button
                                    onClick={() => setDeleteConfirmId(null)}
                                    className="px-1.5 py-0.5 brutal-border bg-gray-200 text-[8px] font-black"
                                  >
                                    Cancel
                                  </button>
                                </div>
                              ) : (
                                <button
                                  onClick={() => setDeleteConfirmId(skill.id)}
                                  className="p-1 brutal-border hover:bg-red-50 text-gray-500"
                                  title="Delete"
                                >
                                  <Trash2 size={12} />
                                </button>
                              )}
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </>
            )}

            {/* Skill form modal */}
            <SkillFormModal
              open={skillModalOpen}
              onClose={() => {
                setSkillModalOpen(false);
                setEditingSkill(null);
              }}
              skill={editingSkill}
              submitting={skillSubmitting}
              onSubmit={async (data) => {
                if (!selectedAgent) return;
                setSkillSubmitting(true);
                try {
                  if (editingSkill) {
                    await updateSkillAction(selectedAgent.agent.agent_id, editingSkill.id, {
                      name: data.name,
                      skill_type: data.skill_type,
                      config: data.config,
                    });
                  } else {
                    await addSkillAction(selectedAgent.agent.agent_id, data.name, data.skill_type, data.config);
                  }
                  setSkillModalOpen(false);
                  setEditingSkill(null);
                } catch {
                  // Error is handled by the hook
                } finally {
                  setSkillSubmitting(false);
                }
              }}
            />
          </div>
        )}

        {activeTab === 'ACTIVITY' && (
          <div className="space-y-4">
            {/* Filter controls */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Filter size={14} className="text-gray-400" />
                <button
                  onClick={() => setActivityAgentFilter(null)}
                  className={cn(
                    "px-2 py-0.5 brutal-border text-[10px] font-black uppercase",
                    !activityAgentFilter ? "bg-brutal-yellow" : "bg-white hover:bg-gray-100"
                  )}
                >
                  All ({activityTotal})
                </button>
                {allAgents.map((awr) => (
                  <button
                    key={awr.agent.agent_id}
                    onClick={() => setActivityAgentFilter(awr.agent.agent_id)}
                    className={cn(
                      "px-2 py-0.5 brutal-border text-[10px] font-black uppercase flex items-center gap-1",
                      activityAgentFilter === awr.agent.agent_id ? "bg-brutal-yellow" : "bg-white hover:bg-gray-100"
                    )}
                  >
                    {awr.agent.emoji} {awr.agent.name}
                  </button>
                ))}
              </div>
              <button
                onClick={refreshActivity}
                className="p-1 brutal-border hover:bg-gray-100"
                title="Refresh"
              >
                <RotateCcw size={12} className={activityLoading ? "animate-spin" : ""} />
              </button>
            </div>

            {/* Activity timeline */}
            {activityLoading && activityEntries.length === 0 ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 size={24} className="animate-spin text-gray-400" />
              </div>
            ) : activityEntries.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-gray-400">
                <Activity size={48} strokeWidth={1} className="mb-2 opacity-20" />
                <span className="text-sm italic">No activity yet</span>
                <span className="text-[10px] mt-1">Activities will appear here as you interact with agents</span>
              </div>
            ) : (
              <div className="space-y-1 font-mono text-[11px]">
                {activityEntries.map((entry) => {
                  const config = getActivityTypeConfig(entry.activity_type);
                  return (
                    <div
                      key={entry.id}
                      className="flex gap-4 py-1.5 hover:bg-white/50 px-2 transition-colors brutal-border border-transparent hover:border-l-black hover:border-l-2"
                    >
                      <span className="text-gray-400 shrink-0 w-20">
                        {new Date(entry.timestamp).toLocaleTimeString([], {
                          hour: '2-digit',
                          minute: '2-digit',
                          second: '2-digit',
                        })}
                      </span>
                      <div className="flex items-center gap-2 shrink-0 w-32">
                        <Circle size={8} className={config.color} fill="currentColor" />
                        <span className={cn("font-bold text-[10px] px-1", config.color)}>
                          {config.label}
                        </span>
                      </div>
                      <span className="text-gray-600 truncate">{entry.summary}</span>
                      {entry.agent_id && (
                        <span className="text-[9px] text-gray-400 shrink-0 ml-auto">
                          @{entry.agent_id}
                        </span>
                      )}
                    </div>
                  );
                })}
                {hasMoreActivity && (
                  <button
                    onClick={loadMoreActivity}
                    className="w-full py-2 text-[10px] font-bold text-gray-500 hover:text-black text-center hover:bg-gray-100 brutal-border border-transparent hover:border-black"
                  >
                    Load more...
                  </button>
                )}
              </div>
            )}
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
                <div className="flex flex-col items-center py-8 brutal-border bg-white brutal-shadow-sm relative">
                  {/* Edit button */}
                  <button
                    onClick={() => setShowEditAgent(true)}
                    className="absolute top-3 right-3 p-1.5 brutal-border bg-white hover:bg-gray-100 transition-colors"
                    title="Edit Agent"
                  >
                    <Pencil size={14} />
                  </button>
                  <AgentIcon
                    icon={profileData.identity.icon}
                    emoji={profileData.identity.emoji}
                    size="lg"
                    bgColor="bg-brutal-cyan"
                    className="mb-4"
                  />
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
                        {selectedAgent.runtime_type === 'claude_code' ? 'Claude Code'
                          : selectedAgent.runtime_type === 'codex' ? 'Codex'
                          : selectedAgent.runtime_type === 'gemini' ? 'Gemini CLI'
                          : selectedAgent.runtime_type || 'Unknown'}
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

      {/* Mention toast — fixed top-right */}
      {mentionToast && (
        <div className="fixed top-4 right-4 z-[100] animate-slide-in-right">
          <div className={cn(
            "brutal-border brutal-shadow px-4 py-3 flex items-center gap-3 max-w-xs",
            mentionToast === 'self' ? "bg-brutal-yellow" : "bg-purple-400"
          )}>
            <span className={cn(
              "text-[10px] font-black uppercase text-white px-1.5 py-0.5 shrink-0",
              mentionToast === 'self' ? "bg-black" : "bg-black/30"
            )}>
              {mentionToast === 'self' ? 'Self-mention' : '@Mention'}
            </span>
            <span className={cn("text-xs font-bold", mentionToast === 'self' ? "text-black" : "text-white")}>
              {mentionToast === 'self'
                ? 'You mentioned yourself — agents can see this.'
                : `An agent mentioned you (@${userProfile.name}) in #${activeChannel?.name}.`}
            </span>
            <button
              onClick={() => setMentionToast(null)}
              className="ml-auto p-1 hover:bg-black/10 shrink-0"
            >
              <svg width="10" height="10" viewBox="0 0 10 10" className="stroke-current"><line x1="1" y1="1" x2="9" y2="9" strokeWidth="2"/><line x1="9" y1="1" x2="1" y2="9" strokeWidth="2"/></svg>
            </button>
          </div>
        </div>
      )}

      {/* Edit Agent Modal */}
      <EditAgentModal
        isOpen={showEditAgent}
        onClose={() => setShowEditAgent(false)}
        onSuccess={() => {
          // Reload profile data after edit
          if (selectedAgent?.agent.agent_id) {
            loadProfile(selectedAgent.agent.agent_id);
          }
        }}
        agentId={selectedAgent?.agent.agent_id ?? null}
      />
    </div>
  );
};
