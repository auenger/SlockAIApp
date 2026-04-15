/**
 * React hook for Task execution engine.
 *
 * Provides execution control (execute, cancel), listens to task://* events
 * for real-time status updates, and tracks active tasks.
 */

import { useState, useEffect, useCallback } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  executeTask,
  cancelTaskExecution,
  getTaskEngineStatus,
} from './ipc';
import type { Task, TaskStatus } from '../types';

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

export interface TaskStatusChangedEvent {
  task_id: string;
  old_status?: string;
  new_status: string;
  mode?: string;
}

export interface TaskCompletedEvent {
  task_id: string;
  result: string;
  agent_id?: string;
}

export interface TaskFailedEvent {
  task_id: string;
  error: string;
  retry_count?: number;
}

export interface TaskCancelledEvent {
  task_id: string;
}

export interface TaskRetryEvent {
  task_id: string;
  retry_count: number;
  max_retry: number;
  error: string;
}

export interface TaskProgressEvent {
  task_id?: string;
  text: string;
}

export interface TaskExecuteRealtimeEvent {
  task_id: string;
  agent_id: string;
  channel_id: string;
  task_prompt: string;
  task_title: string;
}

export interface TaskExecuteAsyncEvent {
  task_id: string;
  agent_id: string;
  channel_id: string;
  task_prompt: string;
  retry_count: number;
}

// ---------------------------------------------------------------------------
// Active task info
// ---------------------------------------------------------------------------

export interface ActiveTaskInfo {
  taskId: string;
  agentId: string;
  channelId: string;
  startedAt: string;
  mode: string;
  progress: string;
}

// ---------------------------------------------------------------------------
// Hook state
// ---------------------------------------------------------------------------

interface TaskEngineState {
  /** Currently active (executing) tasks */
  activeTasks: Map<string, ActiveTaskInfo>;
  /** Async queue length */
  queueLength: number;
  /** Error message (if any) */
  error: string | null;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Hook to manage task execution lifecycle.
 *
 * Listens to all task://* Tauri events and provides methods to
 * execute and cancel tasks.
 */
export function useTaskEngine(
  onStatusChanged?: (event: TaskStatusChangedEvent) => void,
  onCompleted?: (event: TaskCompletedEvent) => void,
  onFailed?: (event: TaskFailedEvent) => void,
) {
  const [state, setState] = useState<TaskEngineState>({
    activeTasks: new Map(),
    queueLength: 0,
    error: null,
  });

  // ---- Event listeners ----
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    async function setupListeners() {
      // task://status-changed
      unlisteners.push(
        await listen<TaskStatusChangedEvent>('task://status-changed', (event) => {
          const data = event.payload;
          setState((prev) => {
            const updated = new Map(prev.activeTasks);
            if (data.new_status === 'in_progress') {
              // Task started — will be added by execute-realtime or execute-async events
            } else if (
              data.new_status === 'in_review' ||
              data.new_status === 'done' ||
              data.new_status === 'cancelled' ||
              data.new_status === 'blocked'
            ) {
              // Task finished — remove from active
              updated.delete(data.task_id);
            }
            return { ...prev, activeTasks: updated };
          });
          onStatusChanged?.(data);
        })
      );

      // task://execute-realtime
      unlisteners.push(
        await listen<TaskExecuteRealtimeEvent>('task://execute-realtime', (event) => {
          const data = event.payload;
          setState((prev) => {
            const updated = new Map(prev.activeTasks);
            updated.set(data.task_id, {
              taskId: data.task_id,
              agentId: data.agent_id,
              channelId: data.channel_id,
              startedAt: new Date().toISOString(),
              mode: 'realtime',
              progress: '',
            });
            return { ...prev, activeTasks: updated };
          });
        })
      );

      // task://execute-async
      unlisteners.push(
        await listen<TaskExecuteAsyncEvent>('task://execute-async', (event) => {
          const data = event.payload;
          setState((prev) => {
            const updated = new Map(prev.activeTasks);
            updated.set(data.task_id, {
              taskId: data.task_id,
              agentId: data.agent_id,
              channelId: data.channel_id,
              startedAt: new Date().toISOString(),
              mode: 'async',
              progress: '',
            });
            return { ...prev, activeTasks: updated };
          });
        })
      );

      // task://completed
      unlisteners.push(
        await listen<TaskCompletedEvent>('task://completed', (event) => {
          const data = event.payload;
          setState((prev) => {
            const updated = new Map(prev.activeTasks);
            updated.delete(data.task_id);
            return { ...prev, activeTasks: updated, error: null };
          });
          onCompleted?.(data);
        })
      );

      // task://failed
      unlisteners.push(
        await listen<TaskFailedEvent>('task://failed', (event) => {
          const data = event.payload;
          setState((prev) => {
            const updated = new Map(prev.activeTasks);
            updated.delete(data.task_id);
            return {
              ...prev,
              activeTasks: updated,
              error: `Task ${data.task_id} failed: ${data.error}`,
            };
          });
          onFailed?.(data);
        })
      );

      // task://cancelled
      unlisteners.push(
        await listen<TaskCancelledEvent>('task://cancelled', (event) => {
          const data = event.payload;
          setState((prev) => {
            const updated = new Map(prev.activeTasks);
            updated.delete(data.task_id);
            return { ...prev, activeTasks: updated, error: null };
          });
        })
      );

      // task://retry
      unlisteners.push(
        await listen<TaskRetryEvent>('task://retry', (event) => {
          const data = event.payload;
          // Keep the task in active but update progress
          setState((prev) => {
            const updated = new Map(prev.activeTasks);
            const existing = updated.get(data.task_id);
            if (existing) {
              updated.set(data.task_id, {
                ...existing,
                progress: `Retrying (${data.retry_count}/${data.max_retry}): ${data.error}`,
              });
            }
            return { ...prev, activeTasks: updated };
          });
        })
      );

      // task://progress
      unlisteners.push(
        await listen<TaskProgressEvent>('task://progress', (event) => {
          const data = event.payload;
          if (data.task_id) {
            setState((prev) => {
              const updated = new Map(prev.activeTasks);
              const existing = updated.get(data.task_id);
              if (existing) {
                updated.set(data.task_id, {
                  ...existing,
                  progress: data.text,
                });
              }
              return { ...prev, activeTasks: updated };
            });
          }
        })
      );
    }

    setupListeners();

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, [onStatusChanged, onCompleted, onFailed]);

  // ---- Actions ----

  /** Execute a task */
  const handleExecute = useCallback(async (taskId: string): Promise<boolean> => {
    try {
      await executeTask(taskId);
      return true;
    } catch (err) {
      setState((prev) => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return false;
    }
  }, []);

  /** Cancel a running task */
  const handleCancel = useCallback(async (taskId: string): Promise<boolean> => {
    try {
      await cancelTaskExecution(taskId);
      return true;
    } catch (err) {
      setState((prev) => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return false;
    }
  }, []);

  /** Refresh engine status */
  const refreshStatus = useCallback(async () => {
    try {
      const status = await getTaskEngineStatus();
      setState((prev) => ({
        ...prev,
        queueLength: status.queue_length,
      }));
    } catch {
      // Ignore — status refresh is best-effort
    }
  }, []);

  /** Check if a task is currently executing */
  const isTaskActive = useCallback(
    (taskId: string): boolean => state.activeTasks.has(taskId),
    [state.activeTasks]
  );

  /** Get active task info */
  const getActiveTask = useCallback(
    (taskId: string): ActiveTaskInfo | undefined => state.activeTasks.get(taskId),
    [state.activeTasks]
  );

  return {
    activeTasks: state.activeTasks,
    queueLength: state.queueLength,
    error: state.error,
    executeTask: handleExecute,
    cancelTask: handleCancel,
    refreshStatus,
    isTaskActive,
    getActiveTask,
  };
}
