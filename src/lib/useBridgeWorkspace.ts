import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  BridgeWorkspaceInfo,
  BridgeAgent,
  BridgeFileEntry,
  BridgeFileContent,
  RemoteConnectionInfo,
} from '../types';

/** Check if a remote connection supports bridge.* operations. */
export function isBridgeEndpoint(conn: RemoteConnectionInfo): boolean {
  if (!conn.agent_card?.supported_operations) return false;
  return conn.agent_card.supported_operations.some((op) => op.startsWith('bridge.'));
}

/** Call a JSON-RPC method on a remote A2A endpoint. */
async function bridgeRpc<T>(
  endpointUrl: string,
  method: string,
  params: Record<string, unknown> = {}
): Promise<T> {
  const response = await fetch(endpointUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
      id: Date.now(),
    }),
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const json = await response.json();
  if (json.error) {
    throw new Error(json.error.message || 'JSON-RPC error');
  }
  return json.result as T;
}

interface BridgeWorkspaceState {
  /** Whether the remote connection is a bridge endpoint. */
  isBridge: boolean;
  /** Workspace info (null if not loaded or not a bridge). */
  workspaceInfo: BridgeWorkspaceInfo | null;
  /** Remote agents. */
  agents: BridgeAgent[];
  /** Whether data is being loaded. */
  loading: boolean;
  /** Error message if any. */
  error: string | null;
  /** Refresh all data from the bridge. */
  refresh: () => Promise<void>;
  /** List files for a specific agent. */
  listFiles: (agentId: string, path?: string) => Promise<BridgeFileEntry[]>;
  /** Read a file from a specific agent's workspace. */
  readFile: (agentId: string, filePath: string) => Promise<BridgeFileContent>;
}

const POLL_INTERVAL = 30_000; // 30s

/**
 * Hook to interact with a remote bridge workspace.
 *
 * Detects if the remote connection is a bridge endpoint (has bridge.* operations),
 * then fetches workspace info and agent list. Supports polling for updates.
 */
export function useBridgeWorkspace(conn: RemoteConnectionInfo | null): BridgeWorkspaceState {
  const [isBridge, setIsBridge] = useState(false);
  const [workspaceInfo, setWorkspaceInfo] = useState<BridgeWorkspaceInfo | null>(null);
  const [agents, setAgents] = useState<BridgeAgent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const refresh = useCallback(async () => {
    if (!conn) return;

    const bridge = isBridgeEndpoint(conn);
    setIsBridge(bridge);
    if (!bridge) return;

    setLoading(true);
    setError(null);
    try {
      const [info, agentsResult] = await Promise.all([
        bridgeRpc<BridgeWorkspaceInfo>(conn.endpoint_url, 'bridge.getWorkspaceInfo'),
        bridgeRpc<{ agents: BridgeAgent[] }>(conn.endpoint_url, 'bridge.getAgents'),
      ]);
      setWorkspaceInfo(info);
      setAgents(agentsResult.agents ?? []);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch bridge data');
    } finally {
      setLoading(false);
    }
  }, [conn]);

  const listFiles = useCallback(
    async (agentId: string, path?: string): Promise<BridgeFileEntry[]> => {
      if (!conn) return [];
      const result = await bridgeRpc<{ entries: BridgeFileEntry[] }>(
        conn.endpoint_url,
        'bridge.listFiles',
        { agent_id: agentId, path }
      );
      return result.entries ?? [];
    },
    [conn]
  );

  const readFile = useCallback(
    async (agentId: string, filePath: string): Promise<BridgeFileContent> => {
      if (!conn) throw new Error('No connection');
      return bridgeRpc<BridgeFileContent>(conn.endpoint_url, 'bridge.readFile', {
        agent_id: agentId,
        file_path: filePath,
      });
    },
    [conn]
  );

  // Load on connection change
  useEffect(() => {
    if (conn) {
      refresh();
    } else {
      setIsBridge(false);
      setWorkspaceInfo(null);
      setAgents([]);
    }
  }, [conn, refresh]);

  // Set up polling
  useEffect(() => {
    if (pollRef.current) clearInterval(pollRef.current);
    if (conn && isBridge) {
      pollRef.current = setInterval(refresh, POLL_INTERVAL);
    }
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [conn, isBridge, refresh]);

  return { isBridge, workspaceInfo, agents, loading, error, refresh, listFiles, readFile };
}
