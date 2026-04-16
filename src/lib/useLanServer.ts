/**
 * React hook for managing the LAN A2A server lifecycle.
 *
 * Provides state management for starting/stopping the A2A TCP server,
 * querying its status, and listing local IP addresses.
 */

import { useState, useEffect, useCallback } from "react";
import type { LanServerInfo } from "../types";
import {
  startA2aServer,
  stopA2aServer,
  getA2aServerStatus,
} from "../lib/ipc";

export interface UseLanServerReturn {
  /** Current server info. */
  serverInfo: LanServerInfo | null;
  /** Whether the server is currently running. */
  isRunning: boolean;
  /** Whether an operation is in progress. */
  loading: boolean;
  /** Error message if any operation failed. */
  error: string | null;
  /** Start the server on the given port. */
  start: (port: number) => Promise<LanServerInfo | null>;
  /** Stop the server. */
  stop: () => Promise<boolean>;
  /** Refresh the server status. */
  refresh: () => Promise<void>;
}

/** Default LAN server info when no server is running. */
const defaultServerInfo: LanServerInfo = {
  status: "stopped",
  port: 0,
  local_ips: [],
  agent_card_url: null,
};

export function useLanServer(): UseLanServerReturn {
  const [serverInfo, setServerInfo] = useState<LanServerInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const info = await getA2aServerStatus();
      setServerInfo(info);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const start = useCallback(
    async (port: number): Promise<LanServerInfo | null> => {
      setLoading(true);
      setError(null);
      try {
        const info = await startA2aServer(port);
        setServerInfo(info);
        return info;
      } catch (e) {
        const msg = String(e);
        setError(msg);
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const stop = useCallback(async (): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      await stopA2aServer();
      setServerInfo(defaultServerInfo);
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const isRunning =
    serverInfo !== null &&
    typeof serverInfo.status === "string" &&
    serverInfo.status === "running";

  return {
    serverInfo: serverInfo ?? defaultServerInfo,
    isRunning,
    loading,
    error,
    start,
    stop,
    refresh,
  };
}
