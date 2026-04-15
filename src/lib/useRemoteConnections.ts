/**
 * React hook for managing remote A2A connections.
 *
 * Provides state management for CRUD operations, health checking,
 * and connection testing of remote A2A endpoints.
 */

import { useState, useEffect, useCallback } from "react";
import type {
  RemoteConnectionInfo,
  CreateRemoteConnectionRequest,
  UpdateRemoteConnectionRequest,
  TestConnectionResult,
} from "../types";
import {
  remoteConnectionList,
  remoteConnectionCreate,
  remoteConnectionUpdate,
  remoteConnectionDelete,
  remoteConnectionTest,
  remoteConnectionHealthAll,
} from "../lib/ipc";

export interface UseRemoteConnectionsReturn {
  /** List of remote connections. */
  connections: RemoteConnectionInfo[];
  /** Whether the list is loading. */
  loading: boolean;
  /** Error message if any operation failed. */
  error: string | null;
  /** Refresh the connection list. */
  refresh: () => Promise<void>;
  /** Create a new connection. */
  create: (request: CreateRemoteConnectionRequest) => Promise<RemoteConnectionInfo | null>;
  /** Update an existing connection. */
  update: (id: string, request: UpdateRemoteConnectionRequest) => Promise<RemoteConnectionInfo | null>;
  /** Delete a connection. */
  remove: (id: string) => Promise<boolean>;
  /** Test a connection (health check). */
  test: (id: string) => Promise<TestConnectionResult | null>;
  /** Health check all connections. */
  healthCheckAll: () => Promise<void>;
  /** Map of connection ID to test result. */
  testResults: Map<string, TestConnectionResult>;
}

export function useRemoteConnections(): UseRemoteConnectionsReturn {
  const [connections, setConnections] = useState<RemoteConnectionInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Map<string, TestConnectionResult>>(new Map());

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await remoteConnectionList();
      setConnections(list);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const create = useCallback(
    async (request: CreateRemoteConnectionRequest): Promise<RemoteConnectionInfo | null> => {
      try {
        setError(null);
        const conn = await remoteConnectionCreate(request);
        await refresh();
        return conn;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [refresh]
  );

  const update = useCallback(
    async (
      id: string,
      request: UpdateRemoteConnectionRequest
    ): Promise<RemoteConnectionInfo | null> => {
      try {
        setError(null);
        const conn = await remoteConnectionUpdate(id, request);
        await refresh();
        return conn;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [refresh]
  );

  const remove = useCallback(
    async (id: string): Promise<boolean> => {
      try {
        setError(null);
        await remoteConnectionDelete(id);
        await refresh();
        return true;
      } catch (e) {
        setError(String(e));
        return false;
      }
    },
    [refresh]
  );

  const test = useCallback(
    async (id: string): Promise<TestConnectionResult | null> => {
      try {
        setError(null);
        const result = await remoteConnectionTest(id);
        setTestResults((prev) => {
          const next = new Map(prev);
          next.set(id, result);
          return next;
        });
        await refresh();
        return result;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [refresh]
  );

  const healthCheckAll = useCallback(async () => {
    try {
      setError(null);
      const updated = await remoteConnectionHealthAll();
      setConnections(updated);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return {
    connections,
    loading,
    error,
    refresh,
    create,
    update,
    remove,
    test,
    healthCheckAll,
    testResults,
  };
}
