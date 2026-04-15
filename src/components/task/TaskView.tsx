/**
 * TaskView — Global Task view container with Board/List toggle, search, and filters.
 *
 * Serves as the main entry point for the Task UI, wiring together
 * TaskBoard, TaskList, TaskDetail, and filter controls.
 */

import React, { useState, useCallback, useMemo } from 'react';
import { Search, LayoutGrid, List, Filter } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTasks, type TaskFilters } from '../../lib/useTasks';
import { useTaskEngine } from '../../lib/useTaskEngine';
import { TaskBoard } from './TaskBoard';
import { TaskList } from './TaskList';
import { TaskDetail } from './TaskDetail';
import { TaskCreateModal, type TaskFormData } from './TaskCreateModal';
import type { TaskStatus, AgentWithRuntime, CreateTaskInput } from '../../types';

// ---------------------------------------------------------------------------
// View mode
// ---------------------------------------------------------------------------

type ViewMode = 'board' | 'list';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TaskViewProps {
  /** Available agents */
  agents: AgentWithRuntime[];
  /** Optional channel ID to filter tasks */
  channelId?: string;
  /** User profile name for creator */
  userName?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskView: React.FC<TaskViewProps> = ({
  agents,
  channelId,
  userName,
}) => {
  const [viewMode, setViewMode] = useState<ViewMode>('board');
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<TaskStatus | 'all'>('all');
  const [assigneeFilter, setAssigneeFilter] = useState<string | 'all'>('all');
  const [detailTaskId, setDetailTaskId] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  // Build filters
  const filters: TaskFilters = useMemo(() => ({
    status: statusFilter !== 'all' ? statusFilter : undefined,
    channelId: channelId,
    assigneeId: assigneeFilter !== 'all' ? assigneeFilter : undefined,
  }), [statusFilter, channelId, assigneeFilter]);

  const {
    tasks,
    loading,
    error,
    createTask,
    updateTask,
    deleteTask,
    updateTaskStatus,
    refresh,
  } = useTasks(filters);

  // Task engine for execution
  const {
    isTaskActive,
    executeTask: engineExecute,
    cancelTask: engineCancel,
  } = useTaskEngine(
    // On status changed — refresh list
    useCallback(() => { refresh(); }, [refresh]),
    // On completed — refresh list
    useCallback(() => { refresh(); }, [refresh]),
    // On failed — refresh list
    useCallback(() => { refresh(); }, [refresh]),
  );

  // Filter tasks by search query
  const filteredTasks = useMemo(() => {
    if (!searchQuery.trim()) return tasks;
    const q = searchQuery.toLowerCase();
    return tasks.filter(t =>
      t.title.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q)
    );
  }, [tasks, searchQuery]);

  // Detail task
  const detailTask = detailTaskId ? tasks.find(t => t.id === detailTaskId) ?? null : null;

  // Handlers
  const handleStatusChange = useCallback(async (taskId: string, newStatus: TaskStatus) => {
    await updateTaskStatus(taskId, newStatus);
  }, [updateTaskStatus]);

  const handleCreateTask = useCallback(async (data: TaskFormData) => {
    const input: CreateTaskInput = {
      title: data.title,
      description: data.description || undefined,
      priority: data.priority,
      creatorId: userName || 'user',
      creatorType: 'user',
      assigneeId: data.assigneeId ?? undefined,
      channelId: data.channelId ?? channelId ?? undefined,
      executionMode: data.executionMode,
      source: 'manual',
    };
    await createTask(input);
  }, [createTask, userName, channelId]);

  const handleEditTask = useCallback(async (taskId: string, data: TaskFormData) => {
    await updateTask(taskId, {
      title: data.title,
      description: data.description,
      priority: data.priority,
      assigneeId: data.assigneeId,
      executionMode: data.executionMode,
    });
  }, [updateTask]);

  const handleDeleteTask = useCallback(async (taskId: string) => {
    await deleteTask(taskId);
    if (detailTaskId === taskId) setDetailTaskId(null);
  }, [deleteTask, detailTaskId]);

  const handleBatchDelete = useCallback(async (taskIds: string[]) => {
    for (const id of taskIds) {
      await deleteTask(id);
    }
  }, [deleteTask]);

  const handleExecute = useCallback(async (taskId: string) => {
    await engineExecute(taskId);
  }, [engineExecute]);

  const handleCancelExecution = useCallback(async (taskId: string) => {
    await engineCancel(taskId);
  }, [engineCancel]);

  const statusOptions: (TaskStatus | 'all')[] = ['all', 'todo', 'in_progress', 'in_review', 'done', 'blocked', 'cancelled'];

  return (
    <div className="h-full flex flex-col">
      {/* Header: View toggle + Search + Filters */}
      <div className="flex items-center gap-2 mb-3">
        {/* View toggle */}
        <div className="flex brutal-border">
          <button
            onClick={() => setViewMode('board')}
            className={cn(
              'p-1.5 flex items-center justify-center transition-colors',
              viewMode === 'board' ? 'bg-brutal-yellow' : 'bg-white hover:bg-gray-100'
            )}
            title="Board view"
          >
            <LayoutGrid size={14} />
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={cn(
              'p-1.5 flex items-center justify-center brutal-border-l transition-colors',
              viewMode === 'list' ? 'bg-brutal-yellow' : 'bg-white hover:bg-gray-100'
            )}
            title="List view"
          >
            <List size={14} />
          </button>
        </div>

        {/* Search */}
        <div className="flex-1 relative">
          <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search tasks..."
            className="w-full brutal-border bg-white pl-7 pr-3 py-1.5 text-xs focus:outline-none focus:bg-brutal-bg"
          />
        </div>

        {/* Status filter */}
        <div className="flex items-center gap-1">
          <Filter size={12} className="text-gray-400" />
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as TaskStatus | 'all')}
            className="brutal-border bg-white px-2 py-1 text-[10px] font-black uppercase focus:outline-none"
          >
            {statusOptions.map(s => (
              <option key={s} value={s}>
                {s === 'all' ? 'All Status' : s.replace('_', ' ')}
              </option>
            ))}
          </select>
        </div>

        {/* Assignee filter */}
        <select
          value={assigneeFilter}
          onChange={(e) => setAssigneeFilter(e.target.value)}
          className="brutal-border bg-white px-2 py-1 text-[10px] font-black focus:outline-none"
        >
          <option value="all">All Agents</option>
          {agents.map(a => (
            <option key={a.agent.agent_id} value={a.agent.agent_id}>
              {a.agent.emoji} {a.agent.name}
            </option>
          ))}
        </select>
      </div>

      {/* Error display */}
      {error && (
        <div className="mb-3 p-2 brutal-border bg-red-50 text-red-600 text-xs font-bold">
          Error: {error}
          <button onClick={refresh} className="ml-2 underline">Retry</button>
        </div>
      )}

      {/* Loading state */}
      {loading && tasks.length === 0 ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-gray-400 text-sm italic">Loading tasks...</div>
        </div>
      ) : (
        /* Board or List view */
        viewMode === 'board' ? (
          <TaskBoard
            tasks={filteredTasks}
            agents={agents}
            onStatusChange={handleStatusChange}
            onTaskClick={setDetailTaskId}
            onCreateTask={handleCreateTask}
            channelId={channelId}
          />
        ) : (
          <TaskList
            tasks={filteredTasks}
            agents={agents}
            onTaskClick={setDetailTaskId}
            onDeleteTasks={handleBatchDelete}
            onStatusChange={handleStatusChange}
            onCreateTask={handleCreateTask}
            channelId={channelId}
          />
        )
      )}

      {/* Detail panel */}
      <TaskDetail
        isOpen={!!detailTask}
        onClose={() => setDetailTaskId(null)}
        task={detailTask}
        agents={agents}
        onEdit={handleEditTask}
        onDelete={handleDeleteTask}
        onStatusChange={handleStatusChange}
        onExecute={handleExecute}
        onCancelExecution={handleCancelExecution}
        isExecuting={detailTaskId ? isTaskActive(detailTaskId) : false}
      />

      {/* Standalone create modal (used by external triggers) */}
      <TaskCreateModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onSubmit={handleCreateTask}
        agents={agents}
        channelId={channelId}
      />
    </div>
  );
};
