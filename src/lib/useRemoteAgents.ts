/**
 * React hook for managing remote agent proxies.
 *
 * Provides state management for syncing, listing, and refreshing
 * remote agents from connected A2A bridges.
 */

import { useState, useEffect, useCallback } from "react";
import type { AgentSummary } from "../types";
import {
  syncRemoteAgents,
  getRemoteAgents,
  refreshRemoteAgents,
} from "../lib/ipc";

export interface UseRemoteAgentsReturn {
  /** List of remote agent proxies. */
  remoteAgents: AgentSummary[];
  /** Whether the list is loading. */
  loading: boolean;
  /** Error message if any operation failed. */
  error: string | null;
  /** Refresh the remote agents list. */
  refresh: () => Promise<void>;
  /** Sync agents from a specific connection. */
  sync: (connectionId: string) => Promise<AgentSummary[]>;
  /** Refresh agents for a specific connection (health check + sync). */
  refreshConnection: (connectionId: string) => Promise<void>;
}

export function useRemoteAgents(): UseRemoteAgentsReturn {
  const [remoteAgents, setRemoteAgents] = useState<AgentSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const agents = await getRemoteAgents();
      setRemoteAgents(agents);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const sync = useCallback(async (connectionId: string): Promise<AgentSummary[]> => {
    setError(null);
    try {
      const synced = await syncRemoteAgents(connectionId);
      // Refresh the full list after sync
      await refresh();
      return synced;
    } catch (e) {
      setError(String(e));
      return [];
    }
  }, [refresh]);

  const refreshConnection = useCallback(async (connectionId: string) => {
    setError(null);
    try {
      await refreshRemoteAgents(connectionId);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [refresh]);

  // Load on mount
  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    remoteAgents,
    loading,
    error,
    refresh,
    sync,
    refreshConnection,
  };
}
