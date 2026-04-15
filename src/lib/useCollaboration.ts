/**
 * React hooks for A2A multi-agent collaboration features.
 *
 * Provides:
 * - useCollaboration: delegation management
 * - usePushEvents: subscribe to push notification events
 * - useArtifacts: artifact querying and management
 */

import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  collaborationDelegate,
  collaborationListDelegations,
  collaborationCancelDelegation,
  collaborationRetryDelegation,
  collaborationListArtifacts,
  collaborationGetArtifact,
  collaborationSearchArtifacts,
  collaborationRegisterPushUrl,
  collaborationListPushConfigs,
  collaborationUnregisterPushUrl,
} from "./ipc";
import type {
  DelegationInfo,
  CreateDelegationRequest,
  ArtifactInfo,
  ArtifactContentResult,
  PushNotificationConfigInfo,
  PushEventPayload,
  PushTaskCompletedPayload,
} from "../types";

// ===========================================================================
// useCollaboration
// ===========================================================================

/**
 * Hook for managing task delegations between agents.
 */
export function useCollaboration(agentId?: string) {
  const [delegations, setDelegations] = useState<DelegationInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await collaborationListDelegations({
        agentId,
        activeOnly: false,
      });
      setDelegations(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [agentId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const delegate = useCallback(
    async (request: CreateDelegationRequest): Promise<DelegationInfo> => {
      const result = await collaborationDelegate(request);
      await refresh();
      return result;
    },
    [refresh]
  );

  const cancel = useCallback(
    async (delegationId: string): Promise<void> => {
      await collaborationCancelDelegation(delegationId);
      await refresh();
    },
    [refresh]
  );

  const retry = useCallback(
    async (delegationId: string): Promise<DelegationInfo> => {
      const result = await collaborationRetryDelegation(delegationId);
      await refresh();
      return result;
    },
    [refresh]
  );

  const activeDelegations = delegations.filter(
    (d) => !["COMPLETED", "FAILED", "CANCELLED", "TIMED_OUT"].includes(d.status)
  );

  return {
    delegations,
    activeDelegations,
    loading,
    error,
    delegate,
    cancel,
    retry,
    refresh,
  };
}

// ===========================================================================
// usePushEvents
// ===========================================================================

/**
 * Hook for subscribing to push notification events from A2A agents.
 */
export function usePushEvents() {
  const [events, setEvents] = useState<PushEventPayload[]>([]);
  const [configs, setConfigs] = useState<PushNotificationConfigInfo[]>([]);

  // Subscribe to push events
  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    async function subscribe() {
      const unlisten1 = await listen<PushEventPayload>("a2a://task-updated", (event) => {
        setEvents((prev) => [event.payload, ...prev].slice(0, 50));
      });

      const unlisten2 = await listen<PushTaskCompletedPayload>("a2a://task-completed", () => {
        // Task completed notification handled by task-updated
      });

      const unlisten3 = await listen<PushTaskCompletedPayload>("a2a://task-failed", () => {
        // Task failed notification handled by task-updated
      });

      unlisteners.push(unlisten1, unlisten2, unlisten3);
    }

    subscribe();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  // Load configs
  useEffect(() => {
    collaborationListPushConfigs()
      .then(setConfigs)
      .catch(() => {});
  }, []);

  const registerPushUrl = useCallback(
    async (url: string, token?: string, hmacSecret?: string) => {
      const config = await collaborationRegisterPushUrl({
        url,
        token,
        hmacSecret,
      });
      setConfigs((prev) => [...prev, config]);
      return config;
    },
    []
  );

  const unregisterPushUrl = useCallback(async (configId: string) => {
    await collaborationUnregisterPushUrl(configId);
    setConfigs((prev) => prev.filter((c) => c.id !== configId));
  }, []);

  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  return {
    events,
    configs,
    registerPushUrl,
    unregisterPushUrl,
    clearEvents,
  };
}

// ===========================================================================
// useArtifacts
// ===========================================================================

/**
 * Hook for querying and managing cross-agent artifacts.
 */
export function useArtifacts(agentId?: string) {
  const [artifacts, setArtifacts] = useState<ArtifactInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await collaborationListArtifacts({ agentId });
      setArtifacts(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [agentId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const getArtifactContent = useCallback(
    async (artifactId: string, consumerAgentId?: string): Promise<ArtifactContentResult> => {
      return collaborationGetArtifact(artifactId, consumerAgentId);
    },
    []
  );

  const searchArtifacts = useCallback(async (query: string): Promise<ArtifactInfo[]> => {
    return collaborationSearchArtifacts(query);
  }, []);

  return {
    artifacts,
    loading,
    error,
    getArtifactContent,
    searchArtifacts,
    refresh,
  };
}
