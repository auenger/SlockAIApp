/**
 * RemoteOverviewPanel — displays remote connections and their agents.
 *
 * Shown inside the Sidebar when the Monitor button is toggled.
 * Lists all remote connections with health status indicators,
 * and each connection can be expanded to show its agents.
 */

import React, { useState } from 'react';
import {
  ChevronDown,
  ChevronRight,
  Cloud,
  CloudOff,
  Wifi,
  WifiOff,
  HelpCircle,
  Settings,
  Server,
} from 'lucide-react';
import { cn } from '../lib/utils';
import { AgentIcon } from './AgentIcon';
import { isRemoteAgent, getConnectionId } from '../lib/useAllAgents';
import type {
  RemoteConnectionInfo,
  RemoteConnectionStatus,
  AgentWithRuntime,
} from '../types';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface RemoteOverviewPanelProps {
  /** All remote connections */
  connections: RemoteConnectionInfo[];
  /** All agents (local + remote) — we filter to remote only */
  agents: AgentWithRuntime[];
  /** Connection ID → connection name mapping */
  connectionNames: Map<string, string>;
  /** Whether data is loading */
  loading: boolean;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Status indicator color for connection health. */
function getStatusColor(status: RemoteConnectionStatus): string {
  switch (status) {
    case 'online':
      return '#22c55e'; // green-500
    case 'offline':
    case 'error':
      return '#ef4444'; // red-500
    default:
      return '#9ca3af'; // gray-400
  }
}

/** Status label text. */
function getStatusLabel(status: RemoteConnectionStatus): string {
  switch (status) {
    case 'online':
      return 'Healthy';
    case 'offline':
      return 'Offline';
    case 'error':
      return 'Error';
    default:
      return 'Unknown';
  }
}

/** Group remote agents by connection ID. */
function groupAgentsByConnection(
  agents: AgentWithRuntime[]
): Map<string, AgentWithRuntime[]> {
  const map = new Map<string, AgentWithRuntime[]>();
  for (const awr of agents) {
    if (!isRemoteAgent(awr.agent)) continue;
    const connId = getConnectionId(awr.agent);
    if (!connId) continue;
    const existing = map.get(connId) ?? [];
    existing.push(awr);
    map.set(connId, existing);
  }
  return map;
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

/** Single connection row with expandable agent list. */
const ConnectionRow: React.FC<{
  connection: RemoteConnectionInfo;
  agents: AgentWithRuntime[];
  connectionNames: Map<string, string>;
}> = ({ connection, agents, connectionNames }) => {
  const [expanded, setExpanded] = useState(false);
  const statusColor = getStatusColor(connection.status);
  const agentCount = agents.length;

  return (
    <div className="brutal-border bg-white">
      {/* Connection header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full text-left px-2 py-1.5 flex items-center gap-2 hover:bg-gray-50 transition-colors"
      >
        {expanded ? (
          <ChevronDown size={12} className="shrink-0 text-gray-500" />
        ) : (
          <ChevronRight size={12} className="shrink-0 text-gray-500" />
        )}
        <Server size={12} className="shrink-0 text-gray-600" />
        <span className="font-bold text-[11px] flex-1 truncate">
          {connection.name}
        </span>
        {/* Status dot */}
        <span
          className="w-2 h-2 rounded-full shrink-0"
          style={{ backgroundColor: statusColor }}
          title={getStatusLabel(connection.status)}
        />
        <span className="text-[9px] text-gray-500 font-medium shrink-0">
          {agentCount} {agentCount === 1 ? 'agent' : 'agents'}
        </span>
      </button>

      {/* Agent sub-list */}
      {expanded && (
        <div className="pl-5 pr-2 pb-1.5 space-y-0.5 border-t border-black/10">
          {agents.length > 0 ? (
            agents.map((awr) => {
              const isOffline = awr.runtime_status !== 'available';
              return (
                <div
                  key={awr.agent.agent_id}
                  className={cn(
                    'flex items-center gap-1.5 py-0.5',
                    isOffline && 'opacity-50'
                  )}
                >
                  <div className="relative">
                    <AgentIcon
                      icon={awr.agent.icon}
                      emoji={awr.agent.emoji}
                      size="sm"
                      bgColor="bg-purple-300"
                    />
                    <div
                      className={cn(
                        'absolute -bottom-0.5 -right-0.5',
                        isOffline ? 'text-gray-400' : 'text-purple-500'
                      )}
                    >
                      {isOffline ? (
                        <CloudOff size={7} />
                      ) : (
                        <Cloud size={7} />
                      )}
                    </div>
                  </div>
                  <span className="text-[10px] font-bold truncate flex-1">
                    {awr.agent.name}
                  </span>
                  <span
                    className="w-1.5 h-1.5 rounded-full shrink-0"
                    style={{
                      backgroundColor: isOffline ? '#9ca3af' : '#22c55e',
                    }}
                  />
                </div>
              );
            })
          ) : (
            <div className="text-[9px] text-gray-400 italic py-1">
              No agents synced
            </div>
          )}
        </div>
      )}
    </div>
  );
};

/** Empty state when no remote connections are configured. */
const EmptyState: React.FC = () => (
  <div className="p-3 text-center space-y-2">
    <div className="flex justify-center">
      <div className="w-10 h-10 brutal-border bg-gray-100 flex items-center justify-center">
        <CloudOff size={20} className="text-gray-400" />
      </div>
    </div>
    <div className="text-[11px] font-bold text-gray-600">No Remote Connections</div>
    <div className="text-[9px] text-gray-400 leading-relaxed">
      Add remote machines in{' '}
      <span className="font-bold text-gray-500">Settings &gt; Remote Connections</span>{' '}
      to monitor their agents here.
    </div>
  </div>
);

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export const RemoteOverviewPanel: React.FC<RemoteOverviewPanelProps> = ({
  connections,
  agents,
  connectionNames,
  loading,
}) => {
  // Group remote agents by their connection
  const agentsByConnection = groupAgentsByConnection(agents);

  return (
    <div className="brutal-border-b">
      {/* Section header */}
      <div className="flex items-center justify-between px-2 py-1.5 bg-gray-50 border-b border-black/10">
        <h3 className="font-black text-[10px] uppercase tracking-wider flex items-center gap-1">
          <Wifi size={10} /> Remote Machines
          <span className="text-gray-500">
            {loading ? '...' : connections.length}
          </span>
        </h3>
      </div>

      {/* Content */}
      {loading ? (
        <div className="p-3 text-center">
          <div className="text-[10px] text-gray-400 italic">Loading...</div>
        </div>
      ) : connections.length === 0 ? (
        <EmptyState />
      ) : (
        <div className="p-1 space-y-1">
          {connections.map((conn) => (
            <ConnectionRow
              key={conn.id}
              connection={conn}
              agents={agentsByConnection.get(conn.id) ?? []}
              connectionNames={connectionNames}
            />
          ))}
        </div>
      )}
    </div>
  );
};
