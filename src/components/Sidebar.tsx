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
} from 'lucide-react';
import { cn } from '../lib/utils';
import { useAgentStatus, getRuntimeStatusColor, getRuntimeStatusLabel } from '../lib/useAgentStatus';
import { useUserProfile } from '../lib/useUserProfile';
import { CreateAgentModal } from './CreateAgentModal';
import { EditAgentModal } from './EditAgentModal';
import { ApiKeyManager } from './ApiKeyManager';
import { AgentIcon } from './AgentIcon';
import type { AgentWithRuntime, ThreadInfo, ChannelInfo } from '../types';

interface SidebarProps {
  activeChannel: string;
  onChannelSelect: (id: string) => void;
  /** Currently selected agent ID */
  selectedAgentId: string | null;
  /** Callback when user clicks an agent */
  onAgentSelect: (agent: AgentWithRuntime) => void;
  /** Thread list for the selected agent */
  threads?: ThreadInfo[];
  /** Currently active thread ID */
  activeThreadId?: string | null;
  /** Callback when user clicks a thread */
  onThreadSelect?: (threadId: string) => void;
  /** Callback when user wants to create a new thread */
  onCreateThread?: () => void;
  /** Channel list */
  channels?: ChannelInfo[];
  /** Callback when user wants to create a new channel */
  onCreateChannel?: (name: string, memberAgentIds: string[]) => void;
  /** Available agents for channel member selection */
  agents?: AgentWithRuntime[];
  /** Callback when user wants to delete a channel */
  onDeleteChannel?: (channelId: string) => void;
  /** Callback when user wants to delete a thread */
  onDeleteThread?: (threadId: string) => void;
  /** Callback when user wants to delete an agent */
  onDeleteAgent?: (agentId: string) => void;
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
  channels = [],
  onCreateChannel,
  agents: propAgents,
  onDeleteChannel,
  onDeleteThread,
  onDeleteAgent,
  style,
  resizeHandleRef,
}) => {
  const { agents: statusAgents, loading, scan } = useAgentStatus();
  const { profile } = useUserProfile();
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [showCreateAgent, setShowCreateAgent] = useState(false);
  const [showEditAgent, setShowEditAgent] = useState(false);
  const [editingAgentId, setEditingAgentId] = useState<string | null>(null);
  const [showApiKeyManager, setShowApiKeyManager] = useState(false);
  const [newChannelName, setNewChannelName] = useState('');
  const [selectedMemberIds, setSelectedMemberIds] = useState<string[]>([]);

  // Delete confirmation state: { type, id, name } or null
  const [deleteConfirm, setDeleteConfirm] = useState<{ type: 'channel' | 'thread' | 'agent'; id: string; name: string } | null>(null);

  // Use prop agents if provided (they have runtime status), otherwise use status agents
  const agents = propAgents ?? statusAgents;

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
        <button className="flex-1 p-2 flex justify-center brutal-border-r hover:bg-gray-100">
          <MessageSquare size={20} />
        </button>
        <button className="flex-1 p-2 flex justify-center hover:bg-gray-100">
          <Monitor size={20} />
        </button>
      </div>

      {/* Scrollable Content */}
      <div className="flex-1 overflow-y-auto p-2 space-y-6">
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
                  return (
                    <label
                      key={awr.agent.agent_id}
                      className="flex items-center gap-2 px-1 py-0.5 hover:bg-gray-50 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        checked={checked}
                        onChange={() => toggleMember(awr.agent.agent_id)}
                        className="brutal-border w-3 h-3 accent-black"
                      />
                      <span className="text-[10px] font-bold">{awr.agent.emoji} {awr.agent.name}</span>
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

        {/* Threads - real data from backend */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Threads{' '}
              <span className="text-gray-600">{threads.length}</span>
            </h3>
            {selectedAgentId && onCreateThread && (
              <button
                onClick={onCreateThread}
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
                    <MessageSquare size={12} className="mt-0.5 shrink-0" />
                    <div className="flex-1 min-w-0">
                      <div className={cn(
                        "font-bold text-[11px] truncate",
                        activeThreadId === thread.id ? "text-white" : "text-gray-800"
                      )}>
                        {thread.title}
                      </div>
                      {thread.preview && (
                        <div className={cn(
                          "truncate text-[10px]",
                          activeThreadId === thread.id ? "text-white/80" : "text-gray-500"
                        )}>
                          {thread.preview}
                        </div>
                      )}
                      <div className={cn(
                        "text-[9px] mt-0.5 flex items-center gap-1",
                        activeThreadId === thread.id ? "text-white/60" : "text-gray-400"
                      )}>
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
                {selectedAgentId ? 'No threads yet' : 'Select an agent'}
              </div>
            )}
          </div>
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

              return (
                <div
                  key={agent.agent_id}
                  className="group relative"
                >
                  <button
                    onClick={() => onAgentSelect(agentWithRuntime)}
                    title={
                      runtime_status === 'not-installed' && runtime_install_hint
                        ? `${statusLabel}\nInstall: ${runtime_install_hint}`
                        : statusLabel
                    }
                    className={cn(
                      "w-full text-left px-2 py-1.5 flex items-center gap-2 brutal-border transition-all",
                      isSelected
                        ? "bg-brutal-pink text-white brutal-shadow-sm translate-x-[-2px] translate-y-[-2px]"
                        : "hover:bg-white/50 border-transparent"
                    )}
                  >
                    <AgentIcon
                      icon={agent.icon}
                      emoji={agent.emoji}
                      size="sm"
                      bgColor="bg-brutal-cyan"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="font-black text-sm truncate">{agent.name}</div>
                    </div>
                    <Circle
                      size={8}
                      fill={statusColor}
                      className="shrink-0"
                    />
                  </button>
                  {/* Edit button - visible on hover */}
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
        onSuccess={scan}
      />

      {/* Edit Agent Modal */}
      <EditAgentModal
        isOpen={showEditAgent}
        onClose={() => {
          setShowEditAgent(false);
          setEditingAgentId(null);
        }}
        onSuccess={scan}
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
