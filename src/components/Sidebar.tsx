import React from 'react';
import {
  Hash,
  MessageSquare,
  Plus,
  ChevronDown,
  Monitor,
  Settings,
  User,
  Circle,
} from 'lucide-react';
import { cn } from '../lib/utils';
import { useAgentStatus, getRuntimeStatusColor, getRuntimeStatusLabel } from '../lib/useAgentStatus';
import type { AgentWithRuntime } from '../types';

interface SidebarProps {
  activeChannel: string;
  onChannelSelect: (id: string) => void;
  /** Currently selected agent ID */
  selectedAgentId: string | null;
  /** Callback when user clicks an agent */
  onAgentSelect: (agent: AgentWithRuntime) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeChannel,
  onChannelSelect,
  selectedAgentId,
  onAgentSelect,
}) => {
  const { agents, loading } = useAgentStatus();

  return (
    <div className="w-64 h-full bg-brutal-yellow brutal-border flex flex-col overflow-hidden">
      {/* Header */}
      <div className="p-4 brutal-border-b bg-black text-white flex items-center justify-between">
        <div className="flex items-center gap-2 font-black italic text-lg">
          Development
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
              <ChevronDown size={14} /> Channels <span className="text-gray-600">2</span>
            </h3>
            <button className="brutal-border bg-white p-0.5 hover:bg-gray-100">
              <Plus size={14} />
            </button>
          </div>
          <div className="space-y-1">
            {['all', 'kagent-integrate-sap-ai-core'].map(id => (
              <button
                key={id}
                onClick={() => onChannelSelect(id)}
                className={cn(
                  "w-full text-left px-3 py-1.5 font-bold text-sm flex items-center gap-2 brutal-border transition-all",
                  activeChannel === id ? "bg-brutal-pink text-white brutal-shadow-sm translate-x-[-2px] translate-y-[-2px]" : "hover:bg-white/50 border-transparent"
                )}
              >
                <Hash size={14} /> {id}
              </button>
            ))}
          </div>
        </section>

        {/* Threads */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Threads <span className="text-gray-600">2</span>
            </h3>
          </div>
          <div className="space-y-2">
            {[
              { id: 't1', text: '@Alice 或许我们可以把架构设...' },
              { id: 't2', text: '@Alice @克劳德 我们现在要将...' }
            ].map(thread => (
              <button key={thread.id} className="w-full text-left px-3 py-1 text-xs font-medium hover:underline flex items-start gap-2">
                <MessageSquare size={12} className="mt-0.5 shrink-0" />
                <span className="truncate">{thread.text}</span>
              </button>
            ))}
          </div>
        </section>

        {/* Agents */}
        <section>
          <div className="flex items-center justify-between px-2 mb-2">
            <h3 className="font-black text-xs uppercase tracking-wider flex items-center gap-1">
              <ChevronDown size={14} /> Agents{' '}
              <span className="text-gray-600">{loading ? '...' : agents.length}</span>
            </h3>
            <button className="brutal-border bg-white p-0.5 hover:bg-gray-100">
              <Plus size={14} />
            </button>
          </div>
          <div className="space-y-1">
            {agents.map((agentWithRuntime) => {
              const { agent, runtime_status, runtime_install_hint } = agentWithRuntime;
              const isSelected = selectedAgentId === agent.agent_id;
              const statusColor = getRuntimeStatusColor(runtime_status);
              const statusLabel = getRuntimeStatusLabel(runtime_status);

              return (
                <button
                  key={agent.agent_id}
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
                  <div className="w-6 h-6 brutal-border bg-brutal-cyan flex items-center justify-center shrink-0 text-sm">
                    {agent.emoji}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="font-black text-sm truncate">{agent.name}</div>
                  </div>
                  <Circle
                    size={8}
                    fill={statusColor}
                    className="shrink-0"
                  />
                </button>
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
            <button className="brutal-border bg-white p-0.5 hover:bg-gray-100">
              <Plus size={14} />
            </button>
          </div>
          <button className="w-full text-left px-2 py-1.5 flex items-center gap-2 hover:bg-white/50 brutal-border border-transparent">
            <div className="w-6 h-6 brutal-border bg-purple-400 flex items-center justify-center shrink-0">
              <User size={14} />
            </div>
            <div className="font-black text-sm">Lissa <span className="text-gray-600 font-normal">(you)</span></div>
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
            <div className="font-black text-xs">Lissa</div>
            <div className="text-[10px] text-gray-500">huimintai5@gmail...</div>
          </div>
        </div>
        <button className="p-1 brutal-border hover:bg-gray-100">
          <Settings size={16} />
        </button>
      </div>
    </div>
  );
};
