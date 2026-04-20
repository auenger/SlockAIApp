import React, { useEffect, useRef, useCallback } from 'react';
import { Cloud, MessageSquare } from 'lucide-react';
import { AgentIcon } from './AgentIcon';
import { getRuntimeStatusColor, getRuntimeStatusLabel } from '../lib/useAgentStatus';
import { isRemoteAgent, getConnectionId } from '../lib/useAllAgents';
import type { AgentWithRuntime } from '../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface ActiveAgentPanelProps {
  /** All agents with runtime status */
  agents: AgentWithRuntime[];
  /** Callback when user clicks an agent */
  onAgentSelect: (agent: AgentWithRuntime) => void;
  /** Callback to close the panel */
  onClose: () => void;
  /** Connection ID -> connection name mapping (for remote agent labels) */
  connectionNames?: Map<string, string>;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * ActiveAgentPanel — Popup panel showing all available (active) agents.
 *
 * - Filters agents to only show those with runtime_status === 'available'
 * - Each agent entry shows icon, name, and remote indicator
 * - Clicking an agent triggers onAgentSelect and closes the panel
 * - Click-away and ESC key close the panel
 * - Empty state when no agents are available
 */
export const ActiveAgentPanel: React.FC<ActiveAgentPanelProps> = ({
  agents,
  onAgentSelect,
  onClose,
  connectionNames,
}) => {
  const panelRef = useRef<HTMLDivElement>(null);

  // Filter to available agents only
  const activeAgents = agents.filter((a) => a.runtime_status === 'available');

  // Click-away: close when clicking outside the panel
  const handleClickOutside = useCallback(
    (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        onClose();
      }
    },
    [onClose],
  );

  // ESC key: close the panel
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [onClose],
  );

  useEffect(() => {
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleClickOutside, handleKeyDown]);

  const handleAgentClick = (agent: AgentWithRuntime) => {
    onAgentSelect(agent);
    onClose();
  };

  return (
    <div
      ref={panelRef}
      className="absolute left-0 right-0 top-full z-40 bg-white brutal-border brutal-shadow mt-1 max-h-80 overflow-y-auto"
    >
      {/* Header */}
      <div className="sticky top-0 bg-white px-3 py-2 brutal-border-b flex items-center gap-2">
        <MessageSquare size={14} />
        <span className="text-[10px] font-black uppercase tracking-wider">
          Active Agents
        </span>
        <span className="text-[9px] text-gray-500 font-bold">
          {activeAgents.length}
        </span>
      </div>

      {/* Agent list */}
      {activeAgents.length > 0 ? (
        <div className="p-1 space-y-0.5">
          {activeAgents.map((awr) => {
            const { agent, runtime_status } = awr;
            const statusColor = getRuntimeStatusColor(runtime_status);
            const statusLabel = getRuntimeStatusLabel(runtime_status);
            const remote = isRemoteAgent(agent);
            const connId = remote ? getConnectionId(agent) : null;
            const connName = connId ? connectionNames?.get(connId) : null;

            return (
              <button
                key={agent.agent_id}
                onClick={() => handleAgentClick(awr)}
                className="w-full text-left px-3 py-2 flex items-center gap-2 brutal-border border-transparent hover:bg-gray-50 hover:border-black transition-colors"
                title={`${agent.name} — ${remote ? `Remote${connName ? ` (${connName})` : ''}` : 'Local'} — ${statusLabel}`}
              >
                <div className="relative">
                  <AgentIcon
                    icon={agent.icon}
                    emoji={agent.emoji}
                    size="sm"
                    bgColor={remote ? 'bg-purple-300' : 'bg-brutal-cyan'}
                  />
                  {remote && (
                    <div className="absolute -bottom-0.5 -right-0.5 text-purple-500">
                      <Cloud size={8} />
                    </div>
                  )}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="font-black text-xs truncate">{agent.name}</div>
                  {remote && connName && (
                    <div className="text-[9px] font-bold text-purple-500 flex items-center gap-0.5 truncate">
                      <Cloud size={8} />
                      {connName}
                    </div>
                  )}
                </div>
                <div
                  className="w-2 h-2 brutal-border shrink-0"
                  style={{ backgroundColor: statusColor }}
                  title={statusLabel}
                />
              </button>
            );
          })}
        </div>
      ) : (
        /* Empty state */
        <div className="px-4 py-6 text-center">
          <MessageSquare size={20} className="mx-auto text-gray-300 mb-2" />
          <div className="text-[10px] text-gray-500 font-bold uppercase">
            No active agents
          </div>
          <div className="text-[9px] text-gray-400 mt-1">
            Agents will appear here when their runtimes are online
          </div>
        </div>
      )}
    </div>
  );
};
