/**
 * React hook for Task management.
 *
 * Provides CRUD operations, filtering, and Tauri event listeners
 * for the Agent Task system.
 */

import { useState, useEffect, useCallback } from 'react';
import {
  listTasks,
  createTask,
  updateTask,
  deleteTask,
  updateTaskStatus,
  assignTask,
  cancelTask,
} from './ipc';
import type {
  Task,
  TaskStatus,
  CreateTaskInput,
  UpdateTaskInput,
} from '../types';

// ---------------------------------------------------------------------------
// Hook state
// ---------------------------------------------------------------------------

interface TasksState {
  /** Current list of tasks */
  tasks: Task[];
  /** Whether data is currently being loaded */
  loading: boolean;
  /** Error message (if any) */
  error: string | null;
}

// ---------------------------------------------------------------------------
// Filter type
// ---------------------------------------------------------------------------

export interface TaskFilters {
  status?: TaskStatus;
  channelId?: string;
  assigneeId?: string;
  parentTaskId?: string;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Hook to load and manage tasks with optional filters.
 *
 * @param filters - Optional filters to apply when listing tasks
 */
export function useTasks(filters?: TaskFilters) {
  const [state, setState] = useState<TasksState>({
    tasks: [],
    loading: false,
    error: null,
  });

  /** Load tasks from the backend */
  const load = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const tasks = await listTasks({
        statusFilter: filters?.status,
        channelId: filters?.channelId,
        assigneeId: filters?.assigneeId,
        parentTaskId: filters?.parentTaskId,
      });
      setState({ tasks, loading: false, error: null });
    } catch (err) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      }));
    }
  }, [filters?.status, filters?.channelId, filters?.assigneeId, filters?.parentTaskId]);

  /** Create a new task */
  const handleCreate = useCallback(async (input: CreateTaskInput): Promise<Task | null> => {
    try {
      const task = await createTask(input);
      setState(prev => ({ ...prev, tasks: [...prev.tasks, task] }));
      return task;
    } catch (err) {
      setState(prev => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return null;
    }
  }, []);

  /** Update an existing task */
  const handleUpdate = useCallback(async (taskId: string, input: UpdateTaskInput): Promise<Task | null> => {
    try {
      const updated = await updateTask(taskId, input);
      setState(prev => ({
        ...prev,
        tasks: prev.tasks.map(t => (t.id === taskId ? updated : t)),
      }));
      return updated;
    } catch (err) {
      setState(prev => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return null;
    }
  }, []);

  /** Delete a task */
  const handleDelete = useCallback(async (taskId: string): Promise<boolean> => {
    try {
      await deleteTask(taskId);
      setState(prev => ({
        ...prev,
        tasks: prev.tasks.filter(t => t.id !== taskId),
      }));
      return true;
    } catch (err) {
      setState(prev => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return false;
    }
  }, []);

  /** Update task status (for drag-and-drop) */
  const handleUpdateStatus = useCallback(async (taskId: string, status: TaskStatus): Promise<Task | null> => {
    try {
      const updated = await updateTaskStatus(taskId, status);
      setState(prev => ({
        ...prev,
        tasks: prev.tasks.map(t => (t.id === taskId ? updated : t)),
      }));
      return updated;
    } catch (err) {
      setState(prev => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return null;
    }
  }, []);

  /** Assign a task to an agent */
  const handleAssign = useCallback(async (taskId: string, agentId: string | null): Promise<Task | null> => {
    try {
      const updated = await assignTask(taskId, agentId);
      setState(prev => ({
        ...prev,
        tasks: prev.tasks.map(t => (t.id === taskId ? updated : t)),
      }));
      return updated;
    } catch (err) {
      setState(prev => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return null;
    }
  }, []);

  /** Cancel a task */
  const handleCancel = useCallback(async (taskId: string): Promise<Task | null> => {
    try {
      const updated = await cancelTask(taskId);
      setState(prev => ({
        ...prev,
        tasks: prev.tasks.map(t => (t.id === taskId ? updated : t)),
      }));
      return updated;
    } catch (err) {
      setState(prev => ({
        ...prev,
        error: err instanceof Error ? err.message : String(err),
      }));
      return null;
    }
  }, []);

  /** Refresh the task list */
  const refresh = useCallback(() => {
    load();
  }, [load]);

  // Load tasks on mount and when filters change
  useEffect(() => {
    load();
  }, [load]);

  return {
    tasks: state.tasks,
    loading: state.loading,
    error: state.error,
    createTask: handleCreate,
    updateTask: handleUpdate,
    deleteTask: handleDelete,
    updateTaskStatus: handleUpdateStatus,
    assignTask: handleAssign,
    cancelTask: handleCancel,
    refresh,
  };
}
