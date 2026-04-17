/**
 * Unified agent list hook — merges local + remote agents.
 *
 * Combines AgentWithRuntime from useAgentStatus (local agents)
 * with AgentSummary from useRemoteAgents (remote proxy agents),
 * providing a single list for UI consumption.
 */

import { useMemo, useEffect, useRef } from "react";
import type { AgentSummary, AgentWithRuntime, ConnectionMode, RemoteConnectionStatus } from "../types";
import { useAgentStatus } from "./useAgentStatus";
import { useRemoteAgents } from "./useRemoteAgents";
import { useRemoteConnections } from "./useRemoteConnections";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Check if an agent is a remote proxy. */
export function isRemoteAgent(agent: AgentSummary): boolean {
  return typeof agent.connection_mode === "object" && agent.connection_mode !== null && "remote" in agent.connection_mode;
}

/** Extract the connection_id from a remote agent, or null for local. */
export function getConnectionId(agent: AgentSummary): string | null {
  if (isRemoteAgent(agent)) {
    const cm = agent.connection_mode as { remote: { connection_id: string } };
    return cm.remote.connection_id;
  }
  return null;
}

/** Derive the runtime status for a remote agent based on its connection. */
export function getRemoteRuntimeStatus(
  agent: AgentSummary,
  connectionStatuses: Map<string, RemoteConnectionStatus>
): AgentWithRuntime["runtime_status"] {
  const connId = getConnectionId(agent);
  if (!connId) return "unhealthy";
  const status = connectionStatuses.get(connId);
  if (status === "online") return "available";
  if (status === "offline") return "not-installed";
  return "unhealthy";
}

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface UseAllAgentsReturn {
  /** Unified list of local + remote agents with runtime status. */
  allAgents: AgentWithRuntime[];
  /** Local agents only. */
  localAgents: AgentWithRuntime[];
  /** Remote agents only (as AgentWithRuntime). */
  remoteAgents: AgentWithRuntime[];
  /** Connection ID → connection name mapping. */
  connectionNames: Map<string, string>;
  /** Whether any data is loading. */
  loading: boolean;
  /** Refresh all agent data. */
  refresh: () => Promise<void>;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useAllAgents(): UseAllAgentsReturn {
  const { agents: localAgents, loading: localLoading, scan: localScan } = useAgentStatus();
  const { remoteAgents: remoteSummaries, loading: remoteLoading, refresh: refreshRemote } = useRemoteAgents();
  const { connections, refresh: refreshConnections } = useRemoteConnections();

  // Track online connection IDs to detect changes (e.g. after Settings sync)
  const prevOnlineIds = useRef<string>("");
  useEffect(() => {
    const onlineIds = connections
      .filter((c) => c.status === "online")
      .map((c) => c.id)
      .sort()
      .join(",");
    if (onlineIds !== prevOnlineIds.current && onlineIds.length > 0) {
      prevOnlineIds.current = onlineIds;
      refreshRemote();
    }
  }, [connections, refreshRemote]);

  // Build connection ID → status map
  const connectionStatuses = useMemo(() => {
    const map = new Map<string, RemoteConnectionStatus>();
    for (const conn of connections) {
      map.set(conn.id, conn.status);
    }
    return map;
  }, [connections]);

  // Build connection ID → name map
  const connectionNames = useMemo(() => {
    const map = new Map<string, string>();
    for (const conn of connections) {
      map.set(conn.id, conn.name);
    }
    return map;
  }, [connections]);

  // Convert remote AgentSummary[] to AgentWithRuntime[]
  const remoteAgents = useMemo<AgentWithRuntime[]>(() => {
    return remoteSummaries.map((agent) => ({
      agent,
      runtime_status: getRemoteRuntimeStatus(agent, connectionStatuses),
      runtime_type: agent.runtime_type,
    }));
  }, [remoteSummaries, connectionStatuses]);

  // Merge local + remote
  const allAgents = useMemo(() => {
    return [...localAgents, ...remoteAgents];
  }, [localAgents, remoteAgents]);

  const loading = localLoading || remoteLoading;

  const refresh = async () => {
    await Promise.all([localScan(), refreshRemote(), refreshConnections()]);
  };

  return { allAgents, localAgents, remoteAgents, connectionNames, loading, refresh };
}
