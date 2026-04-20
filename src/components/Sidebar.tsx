import React, { useState } from 'react';
import {
  Hash,
  MessageSquare,
  Plus,
  ChevronDown,
  Monitor,
  Settings,
  User,
  Circle,
  Clock,
  X,
  Pencil,
  Trash2,
  CheckSquare,
  Cloud,
  CloudOff,
} from 'lucide-react';
import { cn } from '../lib/utils';
import { useAgentStatus, getRuntimeStatusColor, getRuntimeStatusLabel } from '../lib/useAgentStatus';
import { useUserProfile } from '../lib/useUserProfile';
import { useRemoteConnections } from '../lib/useRemoteConnections';
import { CreateAgentModal } from './CreateAgentModal';
import { EditAgentModal } from './EditAgentModal';
import { ApiKeyManager } from './ApiKeyManager';
import { AgentIcon } from './AgentIcon';
import { AgentBadge } from './AgentBadge';
import { RemoteOverviewPanel } from './RemoteOverviewPanel';
import { ActiveAgentPanel } from './ActiveAgentPanel';
import { isRemoteAgent, getConnectionId } from '../lib/useAllAgents';
import type { AgentWithRuntime, ThreadInfo, ChannelInfo } from '../types';

interface SidebarProps {
  activeChannel: string;
  onChannelSelect: (id: string) => void;
  /** Currently selected agent ID */
  selectedAgentId: string | null;
  /** Callback when user clicks an agent */
  onAgentSelect: (agent: AgentWithRuntime) => void;
  /** Global thread list (all agents) */
  threads?: ThreadInfo[];
  /** Currently active thread ID */
  activeThreadId?: string | null;
  /** Callback when user clicks a thread */
  onThreadSelect?: (threadId: string) => void;
  /** Callback when user wants to create a new thread (selected agent or first agent) */
  onCreateThread?: () => void;
  /** Callback when user wants to create a new thread for a specific agent */
  onCreateThreadWithAgent?: (agentId: string) => void;
  /** Callback when user renames a thread */
  onRenameThread?: (threadId: string, newTitle: string) => Promise<ThreadInfo | null>;
  /** Channel list */
  channels?: ChannelInfo[];
  /** Callback when user wants to create a new channel */
  onCreateChannel?: (name: string, memberAgentIds: string[]) => void;
  /** Available agents for channel member selection */
  agents?: AgentWithRuntime[];
  /** Connection ID → connection name mapping (for remote agent labels) */
  connectionNames?: Map<string, string>;
  /** Callback when user wants to delete a channel */
  onDeleteChannel?: (channelId: string) => void;
  /** Callback when user wants to delete a thread */
  onDeleteThread?: (threadId: string) => void;
  /** Callback when user wants to delete an agent */
  onDeleteAgent?: (agentId: string) => void;
  /** Callback to refresh the agent list from the parent */
  onRefreshAgents?: () => Promise<void>;
  /** Whether the task view is active */
  isTaskViewActive?: boolean;
  /** Callback when user clicks the TASKS navigation entry */
  onTaskViewOpen?: () => void;
  /** Count of incomplete tasks for badge */
  incompleteTaskCount?: number;
  /** Resizable width style from parent */
  style?: React.CSSProperties;
  /** Resize handle ref from parent */
  resizeHandleRef?: React.RefObject<HTMLDivElement | null>;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeChannel,
  onChannelSelect,
  selectedAgentId,
  onAgentSelect,
  threads = [],
  activeThreadId,
  onThreadSelect,
  onCreateThread,
  onCreateThreadWithAgent,
  onRenameThread,
  channels = [],
  onCreateChannel,
  agents: propAgents,
  connectionNames,
  onDeleteChannel,
  onDeleteThread,
  onDeleteAgent,
  onRefreshAgents,
  isTaskViewActive = false,
  onTaskViewOpen,
  incompleteTaskCount = 0,
  style,
  resizeHandleRef,
}) => {
  const { agents: statusAgents, loading, scan: localScan } = useAgentStatus();
  const { profile } = useUserProfile();
  const { connections: remoteConnections, loading: remoteLoading } = useRemoteConnections();
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [showCreateAgent, setShowCreateAgent] = useState(false);
  const [showEditAgent, setShowEditAgent] = useState(false);
  const [editingAgentId, setEditingAgentId] = useState<string | null>(null);
  const [showApiKeyManager, setShowApiKeyManager] = useState(false);
  const [newChannelName, setNewChannelName] = useState('');
  const [selectedMemberIds, setSelectedMemberIds] = useState<string[]>([]);
  // Thread rename inline edit state
  const [editingThreadId, setEditingThreadId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');
  // New thread agent picker state
  const [showNewThreadPicker, setShowNewThreadPicker] = useState(false);
  // Remote overview panel toggle
  const [showRemotePanel, setShowRemotePanel] = useState(false);
  // Active agent panel toggle (MessageSquare button)
  const [showActiveAgentPanel, setShowActiveAgentPanel] = useState(false);

  // Delete confirmation state: { type, id, name } or null
  const [deleteConfirm, setDeleteConfirm] = useState<{ type: 'channel' | 'thread' | 'agent'; id: string; name: string } | null>(null);

  // Use prop agents if provided (they have runtime status), otherwise use status agents
  const agents = propAgents ?? statusAgents;

  /** Refresh agent list: use parent callback if available, otherwise local scan */
  const refreshAgents = onRefreshAgents ?? localScan;

  /** Handle channel creation */
  const handleCreateChannel = () => {
    if (!newChannelName.trim() || !onCreateChannel) return;
    onCreateChannel(newChannelName.trim(), selectedMemberIds);
    setNewChannelName('');
    setSelectedMemberIds([]);
    setShowCreateChannel(false);
  };

  /** Toggle member selection for channel creation */
  const toggleMember = (agentId: string) => {
    setSelectedMemberIds((prev) =>
      prev.includes(agentId)
        ? prev.filter((id) => id !== agentId)
        : [...prev, agentId]
    );
  };

  /** Confirm and execute deletion */
  const confirmDelete = () => {
    if (!deleteConfirm) return;
    const { type, id } = deleteConfirm;
    if (type === 'channel') onDeleteChannel?.(id);
    else if (type === 'thread') onDeleteThread?.(id);
    else if (type === 'agent') onDeleteAgent?.(id);
    setDeleteConfirm(null);
  };

  return (
    <div className="h-full bg-brutal-yellow brutal-border flex flex-col overflow-hidden relative" style={style}>
      {/* Header */}
      <div className="p-4 brutal-border-b bg-black text-white flex items-center justify-between">
        <div className="flex items-center gap-2 font-black italic text-lg tracking-tight">
          AgentsZone
          <ChevronDown size={18} />
        </div>
      </div>

      {/* Navigation Icons */}
      <div className="flex brutal-border-b bg-white">
        <div className="relative flex-1">
          <button
            onClick={() => {
              setShowActiveAgentPanel(!showActiveAgentPanel);
              setShowRemotePanel(false);
            }}
            className={cn(
              "w-full p-2 flex justify-center brutal-border-r transition-colors",
              showActiveAgentPanel
                ? "bg-brutal-pink text-white"
                : "hover:bg-gray-100"
            )}
            title="Active Agents"
          >
            <MessageSquare size={20} />
          </button>
          {showActiveAgentPanel && (
            <ActiveAgentPanel
              agents={agents}
              onAgentSelect={onAgentSelect}
              onClose={() => setShowActiveAgentPanel(false)}
              connectionNames={connectionNames ?? new Map()}
            />
          )}
        </div>
        <button
          onClick={() => {
            setShowRemotePanel(!showRemotePanel);
            setShowActiveAgentPanel(false);
          }}
          className={cn(
            "flex-1 p-2 flex justify-center transition-colors",
            showRemotePanel
              ? "bg-brutal-pink text-white"
              : "hover:bg-gray-100"
          )}
          title="Toggle Remote Overview"
        >
          <Monitor size={20} />
        </button>
      </div>

      {/* Scrollable Content */}
      <div className="flex-1 overflow-y-auto p-2 space-y-6">
        {/* Remote Overview Panel — toggle via Monitor button */}
        {showRemotePanel && (
          <RemoteOverviewPanel
            connections={remoteConnections}
            agents={agents}
            connectionNames={connectionNames ?? new Map()}
            loading={remoteLoading}
          />
        )}

        {/* Channels */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Channels <span className="text-gray-600">{channels.length}</span>
            </h3>
            <button
              onClick={() => setShowCreateChannel(true)}
              className="brutal-border bg-white p-0.5 hover:bg-gray-100"
              title="Create Channel"
            >
              <Plus size={14} />
            </button>
          </div>
          <div className="space-y-1">
            {channels.map((ch) => (
              <div key={ch.id} className="group relative">
                <button
                  onClick={() => onChannelSelect(ch.id)}
                  className={cn(
                    "w-full text-left px-3 py-1.5 font-bold text-sm flex items-center gap-2 brutal-border transition-all",
                    activeChannel === ch.id ? "bg-brutal-pink text-white brutal-shadow-sm translate-x-[-2px] translate-y-[-2px]" : "hover:bg-white/50 border-transparent"
                  )}
                >
                  <Hash size={14} /> {ch.name}
                  {ch.unread_count > 0 && (
                    <span className="ml-auto text-[9px] bg-brutal-pink text-white px-1 brutal-border">
                      {ch.unread_count}
                    </span>
                  )}
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setDeleteConfirm({ type: 'channel', id: ch.id, name: ch.name });
                  }}
                  className="absolute right-1 top-1/2 -translate-y-1/2 p-0.5 brutal-border bg-white hover:bg-red-50 opacity-0 group-hover:opacity-100 transition-opacity"
                  title="Delete Channel"
                >
                  <Trash2 size={10} />
                </button>
              </div>
            ))}
            {channels.length === 0 && (
              <div className="text-[10px] text-gray-500 italic px-2 py-1">
                No channels yet
              </div>
            )}
          </div>

          {/* Create Channel Form */}
          {showCreateChannel && (
            <div className="mt-2 p-2 bg-white brutal-border space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-black uppercase">New Channel</span>
                <button
                  onClick={() => setShowCreateChannel(false)}
                  className="p-0.5 hover:bg-gray-100"
                >
                  <X size={12} />
                </button>
              </div>
              <input
                type="text"
                value={newChannelName}
                onChange={(e) => setNewChannelName(e.target.value)}
                placeholder="Channel name"
                className="w-full brutal-border px-2 py-1 text-xs focus:outline-none focus:bg-brutal-bg"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleCreateChannel();
                }}
              />
              <div className="space-y-1">
                <span className="text-[9px] font-bold uppercase text-gray-500">Members</span>
                {agents.map((awr) => {
                  const checked = selectedMemberIds.includes(awr.agent.agent_id);
                  const remote = isRemoteAgent(awr.agent);
                  const connId = remote ? getConnectionId(awr.agent) : null;
                  const connName = connId ? connectionNames?.get(connId) : null;
                  const isOfflineRemote = remote && awr.runtime_status !== 'available';
                  return (
                    <label
                      key={awr.agent.agent_id}
                      className={cn(
                        "flex items-center gap-2 px-1 py-0.5 hover:bg-gray-50 cursor-pointer",
                        isOfflineRemote && "opacity-50"
                      )}
                    >
                      <input
                        type="checkbox"
                        checked={checked}
                        onChange={() => toggleMember(awr.agent.agent_id)}
                        disabled={isOfflineRemote}
                        className="brutal-border w-3 h-3 accent-black"
                      />
                      <div className="relative">
                        <AgentIcon
                          icon={awr.agent.icon}
                          emoji={awr.agent.emoji}
                          size="sm"
                          bgColor={remote ? "bg-purple-300" : "bg-brutal-cyan"}
                        />
                        {remote && (
                          <div className={cn(
                            "absolute -bottom-0.5 -right-0.5",
                            isOfflineRemote ? "text-gray-400" : "text-purple-500"
                          )}>
                            {isOfflineRemote ? <CloudOff size={8} /> : <Cloud size={8} />}
                          </div>
                        )}
                      </div>
                      <span className="text-[10px] font-bold">{awr.agent.name}</span>
                      {remote && (
                        <span className={cn(
                          "text-[9px] font-bold flex items-center gap-0.5",
                          isOfflineRemote ? "text-gray-400" : "text-purple-500"
                        )}>
                          <Cloud size={8} />
                          {connName ?? 'remote'}
                        </span>
                      )}
                    </label>
                  );
                })}
              </div>
              <button
                onClick={handleCreateChannel}
                disabled={!newChannelName.trim()}
                className={cn(
                  "w-full brutal-btn text-[10px] font-black",
                  newChannelName.trim() ? "bg-brutal-pink text-white" : "bg-gray-200 text-gray-400 cursor-not-allowed"
                )}
              >
                Create Channel
              </button>
            </div>
          )}
        </section>

        {/* Tasks navigation entry */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Tasks
            </h3>
          </div>
          <button
            onClick={() => onTaskViewOpen?.()}
            className={cn(
              "w-full text-left px-3 py-1.5 font-bold text-sm flex items-center gap-2 brutal-border transition-all",
              isTaskViewActive
                ? "bg-brutal-pink text-white brutal-shadow-sm translate-x-[-2px] translate-y-[-2px]"
                : "hover:bg-white/50 border-transparent"
            )}
          >
            <CheckSquare size={14} />
            <span className="flex-1">Board</span>
            {incompleteTaskCount > 0 && (
              <span className="text-[9px] bg-brutal-pink text-white px-1 brutal-border">
                {incompleteTaskCount}
              </span>
            )}
          </button>
        </section>

        {/* Threads - global list (all agents) */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Threads{' '}
              <span className="text-gray-600">{threads.length}</span>
            </h3>
            {(selectedAgentId || agents.length > 0) && (
              <button
                onClick={() => {
                  if (selectedAgentId) {
                    onCreateThread?.();
                  } else {
                    setShowNewThreadPicker(true);
                  }
                }}
                className="brutal-border bg-white p-0.5 hover:bg-gray-100"
                title="New Thread"
              >
                <Plus size={14} />
              </button>
            )}
          </div>
          <div className="space-y-2">
            {threads.length > 0 ? (
              threads.map((thread) => (
                <div key={thread.id} className="group relative">
                  <button
                    onClick={() => onThreadSelect?.(thread.id)}
                    className={cn(
                      "w-full text-left px-3 py-1.5 text-xs font-medium flex items-start gap-2 transition-all",
                      activeThreadId === thread.id
                        ? "bg-brutal-pink text-white brutal-shadow-sm translate-x-[-2px] translate-y-[-2px]"
                        : "hover:bg-white/50"
                    )}
                  >
                    <AgentIcon
                      icon={thread.agent_icon ?? null}
                      emoji={thread.agent_emoji || 'B'}
                      size="sm"
                      bgColor="bg-brutal-cyan"
                    />
                    <div className="flex-1 min-w-0">
                      {editingThreadId === thread.id ? (
                        <input
                          type="text"
                          value={editingTitle}
                          onChange={(e) => setEditingTitle(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') {
                              e.preventDefault();
                              const newTitle = editingTitle.trim();
                              if (newTitle && newTitle !== thread.title) {
                                onRenameThread?.(thread.id, newTitle);
                              }
                              setEditingThreadId(null);
                            } else if (e.key === 'Escape') {
                              setEditingThreadId(null);
                            }
                          }}
                          onBlur={() => {
                            const newTitle = editingTitle.trim();
                            if (newTitle && newTitle !== thread.title) {
                              onRenameThread?.(thread.id, newTitle);
                            }
                            setEditingThreadId(null);
                          }}
                          onClick={(e) => e.stopPropagation()}
                          className={cn(
                            "w-full px-1 text-[11px] font-bold brutal-border focus:outline-none focus:bg-brutal-bg",
                            activeThreadId === thread.id ? "bg-brutal-pink text-white" : "bg-white text-gray-800"
                          )}
                          autoFocus
                        />
                      ) : (
                        <div
                          className={cn(
                            "font-bold text-[11px] truncate",
                            activeThreadId === thread.id ? "text-white" : "text-gray-800"
                          )}
                          onDoubleClick={(e) => {
                            e.stopPropagation();
                            setEditingThreadId(thread.id);
                            setEditingTitle(thread.title);
                          }}
                        >
                          {thread.title}
                        </div>
                      )}
                      {/* Agent name label */}
                      <div className={cn(
                        "text-[9px] flex items-center gap-1",
                        activeThreadId === thread.id ? "text-white/60" : "text-gray-400"
                      )}>
                        <span className="font-medium">{thread.agent_name || thread.agent_id}</span>
                        <Clock size={8} />
                        {thread.updated_at && new Date(thread.updated_at).toLocaleDateString(undefined, {
                          month: 'short',
                          day: 'numeric',
                          hour: '2-digit',
                          minute: '2-digit',
                        })}
                        <span className="ml-1">{thread.message_count} msg{thread.message_count !== 1 ? 's' : ''}</span>
                      </div>
                    </div>
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setDeleteConfirm({ type: 'thread', id: thread.id, name: thread.title });
                    }}
                    className="absolute right-1 top-1.5 p-0.5 brutal-border bg-white hover:bg-red-50 opacity-0 group-hover:opacity-100 transition-opacity"
                    title="Delete Thread"
                  >
                    <Trash2 size={10} />
                  </button>
                </div>
              ))
            ) : (
              <div className="text-[10px] text-gray-500 italic px-2 py-1">
                No threads yet
              </div>
            )}
          </div>

          {/* New Thread Agent Picker */}
          {showNewThreadPicker && (
            <div className="mt-2 p-2 bg-white brutal-border space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-black uppercase">New Thread — Select Agent</span>
                <button
                  onClick={() => setShowNewThreadPicker(false)}
                  className="p-0.5 hover:bg-gray-100"
                >
                  <X size={12} />
                </button>
              </div>
              <div className="space-y-1 max-h-40 overflow-y-auto">
                {agents.map((awr) => {
                  const remote = isRemoteAgent(awr.agent);
                  const connId = remote ? getConnectionId(awr.agent) : null;
                  const connName = connId ? connectionNames?.get(connId) : null;
                  const isOfflineRemote = remote && awr.runtime_status !== 'available';
                  return (
                    <button
                      key={awr.agent.agent_id}
                      onClick={() => {
                        if (!isOfflineRemote) {
                          onCreateThreadWithAgent?.(awr.agent.agent_id);
                          setShowNewThreadPicker(false);
                        }
                      }}
                      disabled={isOfflineRemote}
                      className={cn(
                        "w-full text-left flex items-center gap-2 px-2 py-1 brutal-border border-transparent transition-colors",
                        isOfflineRemote
                          ? "opacity-40 cursor-not-allowed"
                          : "hover:bg-gray-50 hover:border-black"
                      )}
                    >
                      <div className="relative">
                        <AgentIcon
                          icon={awr.agent.icon}
                          emoji={awr.agent.emoji}
                          size="sm"
                          bgColor={remote ? "bg-purple-300" : "bg-brutal-cyan"}
                        />
                        {remote && (
                          <div className={cn(
                            "absolute -bottom-0.5 -right-0.5",
                            isOfflineRemote ? "text-gray-400" : "text-purple-500"
                          )}>
                            {isOfflineRemote ? <CloudOff size={8} /> : <Cloud size={8} />}
                          </div>
                        )}
                      </div>
                      <span className="text-[10px] font-bold">{awr.agent.name}</span>
                      {remote && (
                        <span className={cn(
                          "text-[9px] font-bold flex items-center gap-0.5",
                          isOfflineRemote ? "text-gray-400" : "text-purple-500"
                        )}>
                          {isOfflineRemote ? <CloudOff size={8} /> : <Cloud size={8} />}
                          {isOfflineRemote ? 'offline' : connName ?? 'remote'}
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>
            </div>
          )}
        </section>

        {/* Agents */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Agents{' '}
              <span className="text-gray-600">{loading ? '...' : agents.length}</span>
            </h3>
            <button
              onClick={() => setShowCreateAgent(true)}
              className="brutal-border bg-white p-0.5 hover:bg-gray-100"
            >
              <Plus size={14} />
            </button>
          </div>
          <div className="space-y-1">
            {agents.map((agentWithRuntime: AgentWithRuntime) => {
              const { agent, runtime_status, runtime_install_hint } = agentWithRuntime;
              const isSelected = selectedAgentId === agent.agent_id;
              const statusColor = getRuntimeStatusColor(runtime_status);
              const statusLabel = getRuntimeStatusLabel(runtime_status);
              const remote = isRemoteAgent(agent);
              const connId = remote ? getConnectionId(agent) : null;
              const connName = connId ? connectionNames?.get(connId) : null;
              const isOfflineRemote = remote && runtime_status !== 'available';

              return (
                <div
                  key={agent.agent_id}
                  className={cn("group relative", isOfflineRemote && "opacity-60")}
                >
                  <button
                    onClick={() => onAgentSelect(agentWithRuntime)}
                    title={
                      runtime_status === 'not-installed' && runtime_install_hint
                        ? `${statusLabel}\nInstall: ${runtime_install_hint}`
                        : remote
                          ? `Remote${connName ? ` (${connName})` : ''} — ${statusLabel}`
                          : statusLabel
                    }
                    className={cn(
                      "w-full text-left px-2 py-1.5 flex items-center gap-2 brutal-border transition-all",
                      isSelected
                        ? "bg-brutal-pink text-white brutal-shadow-sm translate-x-[-2px] translate-y-[-2px]"
                        : "hover:bg-white/50 border-transparent"
                    )}
                  >
                    <div className="relative">
                      <AgentIcon
                        icon={agent.icon}
                        emoji={agent.emoji}
                        size="sm"
                        bgColor={remote ? "bg-purple-300" : "bg-brutal-cyan"}
                      />
                      {remote && (
                        <div className={cn(
                          "absolute -bottom-0.5 -right-0.5 p-0",
                          isOfflineRemote ? "text-gray-400" : "text-purple-500"
                        )}>
                          {isOfflineRemote ? <CloudOff size={8} /> : <Cloud size={8} />}
                        </div>
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="font-black text-sm truncate">{agent.name}</div>
                      {remote && connName && (
                        <div className={cn(
                          "text-[10px] font-bold truncate mt-0.5 flex items-center gap-1",
                          isSelected ? "text-white/70" : "text-purple-500"
                        )}>
                          <Cloud size={9} />
                          {connName}
                        </div>
                      )}
                    </div>
                    <Circle
                      size={8}
                      fill={statusColor}
                      className="shrink-0"
                    />
                  </button>
                  {/* Edit button - only for local agents */}
                  {!remote && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setEditingAgentId(agent.agent_id);
                        setShowEditAgent(true);
                      }}
                      className="absolute right-7 top-1/2 -translate-y-1/2 p-0.5 brutal-border bg-white hover:bg-gray-100 opacity-0 group-hover:opacity-100 transition-opacity"
                      title="Edit Agent"
                    >
                      <Pencil size={10} />
                    </button>
                  )}
                  {/* Delete button - visible on hover */}
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setDeleteConfirm({ type: 'agent', id: agent.agent_id, name: agent.name });
                    }}
                    className="absolute right-1 top-1/2 -translate-y-1/2 p-0.5 brutal-border bg-white hover:bg-red-50 opacity-0 group-hover:opacity-100 transition-opacity"
                    title="Delete Agent"
                  >
                    <Trash2 size={10} />
                  </button>
                </div>
              );
            })}
            {agents.length === 0 && !loading && (
              <div className="text-[10px] text-gray-500 italic px-2 py-1">
                No agents found
              </div>
            )}
          </div>
        </section>

        {/* Humans */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Humans <span className="text-gray-600">1</span>
            </h3>
          </div>
          <button className="w-full text-left px-2 py-1.5 flex items-center gap-2 hover:bg-white/50 brutal-border border-transparent">
            <div className="w-6 h-6 brutal-border bg-purple-400 flex items-center justify-center shrink-0">
              <User size={14} />
            </div>
            <div className="font-black text-sm">{profile.name} <span className="text-gray-600 font-normal">(you)</span></div>
          </button>
        </section>
      </div>

      {/* User Footer */}
      <div className="p-3 brutal-border-t bg-white flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 brutal-border bg-purple-400 flex items-center justify-center">
            <User size={18} />
          </div>
          <div>
            <div className="font-black text-xs">{profile.name}</div>
            <div className="text-[10px] text-gray-500">{profile.email || 'No email set'}</div>
          </div>
        </div>
        <button className="p-1 brutal-border hover:bg-gray-100" onClick={() => setShowApiKeyManager(true)}>
          <Settings size={16} />
        </button>
      </div>

      {/* Create Agent Modal */}
      <CreateAgentModal
        isOpen={showCreateAgent}
        onClose={() => setShowCreateAgent(false)}
        onSuccess={refreshAgents}
      />

      {/* Edit Agent Modal */}
      <EditAgentModal
        isOpen={showEditAgent}
        onClose={() => {
          setShowEditAgent(false);
          setEditingAgentId(null);
        }}
        onSuccess={refreshAgents}
        agentId={editingAgentId}
      />

      {/* API Key Manager Modal */}
      <ApiKeyManager
        isOpen={showApiKeyManager}
        onClose={() => setShowApiKeyManager(false)}
      />

      {/* Delete Confirmation Dialog */}
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
                onClick={confirmDelete}
                className="px-3 py-1 brutal-border bg-brutal-pink text-white text-xs font-black hover:bg-red-500"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Resize Handle — right edge */}
      <div
        ref={resizeHandleRef}
        className="absolute top-0 right-0 bottom-0 w-1 cursor-col-resize hover:bg-black/20 active:bg-black/30 transition-colors z-10"
      />
    </div>
  );
};
