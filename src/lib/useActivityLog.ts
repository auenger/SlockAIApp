/**
 * React hook for Activity Log data.
 *
 * Loads activity entries from the backend with optional agent filter
 * and pagination support.
 */

import { useState, useEffect, useCallback } from 'react';
import { listActivities, clearActivities } from './ipc';
import type { ActivityLogEntry, ActivityType } from '../types';

// ---------------------------------------------------------------------------
// Hook state
// ---------------------------------------------------------------------------

interface ActivityLogState {
  /** Activity entries (newest first) */
  entries: ActivityLogEntry[];
  /** Total count of matching entries */
  total: number;
  /** Whether data is currently being loaded */
  loading: boolean;
  /** Error message (if any) */
  error: string | null;
}

// ---------------------------------------------------------------------------
// Activity type visual config
// ---------------------------------------------------------------------------

/** Configuration for rendering activity types with icons and colors */
export interface ActivityTypeConfig {
  color: string;
  bgColor: string;
  label: string;
}

/** Get visual config for an activity type */
export function getActivityTypeConfig(type: ActivityType): ActivityTypeConfig {
  switch (type) {
    case 'agent_created':
      return { color: 'text-brutal-green', bgColor: 'bg-brutal-green', label: 'Agent Created' };
    case 'agent_deleted':
      return { color: 'text-brutal-pink', bgColor: 'bg-brutal-pink', label: 'Agent Deleted' };
    case 'conversation_started':
      return { color: 'text-brutal-cyan', bgColor: 'bg-brutal-cyan', label: 'Chat Started' };
    case 'conversation_ended':
      return { color: 'text-gray-500', bgColor: 'bg-gray-400', label: 'Chat Ended' };
    case 'skill_changed':
      return { color: 'text-brutal-yellow', bgColor: 'bg-brutal-yellow', label: 'Skill Changed' };
    case 'channel_created':
      return { color: 'text-brutal-green', bgColor: 'bg-brutal-green', label: 'Channel Created' };
    case 'channel_updated':
      return { color: 'text-brutal-cyan', bgColor: 'bg-brutal-cyan', label: 'Channel Updated' };
    case 'channel_deleted':
      return { color: 'text-brutal-pink', bgColor: 'bg-brutal-pink', label: 'Channel Deleted' };
    case 'channel_message':
      return { color: 'text-brutal-yellow', bgColor: 'bg-brutal-yellow', label: 'Channel Message' };
    case 'system':
    default:
      return { color: 'text-gray-600', bgColor: 'bg-gray-300', label: 'System' };
  }
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Hook to load and manage activity log entries.
 *
 * @param agentId - Optional agent ID to filter by
 * @param pageSize - Number of entries per page (default 50)
 */
export function useActivityLog(agentId?: string | null, pageSize = 50) {
  const [state, setState] = useState<ActivityLogState>({
    entries: [],
    total: 0,
    loading: false,
    error: null,
  });

  const [page, setPage] = useState(0);

  /** Load activities from the backend */
  const load = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const result = await listActivities({
        agent_id: agentId ?? null,
        offset: page * pageSize,
        limit: pageSize,
      });
      setState({
        entries: result.entries,
        total: result.total,
        loading: false,
        error: null,
      });
    } catch (err) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      }));
    }
  }, [agentId, page, pageSize]);

  /** Clear all activities */
  const clear = useCallback(async () => {
    try {
      await clearActivities();
      setState(prev => ({ ...prev, entries: [], total: 0 }));
    } catch (err) {
      setState(prev => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
    }
  }, []);

  /** Refresh (reload current page) */
  const refresh = useCallback(() => {
    load();
  }, [load]);

  /** Load more (next page) */
  const loadMore = useCallback(() => {
    if ((page + 1) * pageSize < state.total) {
      setPage(prev => prev + 1);
    }
  }, [page, pageSize, state.total]);

  /** Reset to first page */
  const resetPage = useCallback(() => {
    setPage(0);
  }, []);

  // Reload when dependencies change
  useEffect(() => {
    load();
  }, [load]);

  return {
    entries: state.entries,
    total: state.total,
    loading: state.loading,
    error: state.error,
    page,
    pageSize,
    hasMore: (page + 1) * pageSize < state.total,
    load,
    refresh,
    clear,
    loadMore,
    resetPage,
  };
}
