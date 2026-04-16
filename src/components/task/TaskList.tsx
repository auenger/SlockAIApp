/**
 * TaskList — Table/list view for Tasks.
 *
 * Supports multi-select, batch operations (status change, delete),
 * search, and column sorting.
 */

import React, { useState, useMemo } from 'react';
import { Plus, Trash2, ChevronUp, ChevronDown, CheckSquare } from 'lucide-react';
import { TaskListRow } from './TaskCard';
import { TaskCreateModal, type TaskFormData } from './TaskCreateModal';
import type { Task, TaskStatus, AgentWithRuntime } from '../../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TaskListProps {
  /** All tasks */
  tasks: Task[];
  /** Available agents */
  agents: AgentWithRuntime[];
  /** Callback when a task is clicked */
  onTaskClick: (taskId: string) => void;
  /** Callback to delete tasks */
  onDeleteTasks: (taskIds: string[]) => Promise<void>;
  /** Callback to update status */
  onStatusChange: (taskId: string, newStatus: TaskStatus) => Promise<void>;
  /** Callback to create a new task */
  onCreateTask: (data: TaskFormData) => Promise<void>;
  /** Channel ID for context */
  channelId?: string;
  /** Set of task IDs currently being executed */
  executingTaskIds?: Set<string>;
}

// ---------------------------------------------------------------------------
// Sort field type
// ---------------------------------------------------------------------------

type SortField = 'title' | 'status' | 'priority' | 'createdAt' | 'updatedAt';
type SortDir = 'asc' | 'desc';

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskList: React.FC<TaskListProps> = ({
  tasks,
  agents,
  onTaskClick,
  onDeleteTasks,
  onStatusChange,
  onCreateTask,
  channelId,
  executingTaskIds,
}) => {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [sortField, setSortField] = useState<SortField>('createdAt');
  const [sortDir, setSortDir] = useState<SortDir>('desc');
  const [showCreateModal, setShowCreateModal] = useState(false);

  // Sort tasks
  const sortedTasks = useMemo(() => {
    const sorted = [...tasks].sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case 'title':
          cmp = a.title.localeCompare(b.title);
          break;
        case 'status':
          cmp = a.status.localeCompare(b.status);
          break;
        case 'priority':
          cmp = a.priority - b.priority;
          break;
        case 'createdAt':
          cmp = a.createdAt.localeCompare(b.createdAt);
          break;
        case 'updatedAt':
          cmp = a.updatedAt.localeCompare(b.updatedAt);
          break;
      }
      return sortDir === 'asc' ? cmp : -cmp;
    });
    return sorted;
  }, [tasks, sortField, sortDir]);

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir(d => d === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDir('asc');
    }
  };

  const toggleSelect = (id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selectedIds.size === tasks.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(tasks.map(t => t.id)));
    }
  };

  const handleBatchDelete = async () => {
    if (selectedIds.size === 0) return;
    await onDeleteTasks(Array.from(selectedIds));
    setSelectedIds(new Set());
  };

  const handleBatchStatus = async (status: TaskStatus) => {
    for (const id of selectedIds) {
      await onStatusChange(id, status);
    }
    setSelectedIds(new Set());
  };

  const SortIcon = ({ field }: { field: SortField }) => (
    sortField === field ? (
      sortDir === 'asc' ? <ChevronUp size={10} /> : <ChevronDown size={10} />
    ) : null
  );

  return (
    <div className="h-full flex flex-col">
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-3 gap-2">
        {/* Batch actions */}
        <div className="flex items-center gap-1">
          {selectedIds.size > 0 && (
            <>
              <span className="text-[10px] font-bold text-gray-500 mr-1">
                {selectedIds.size} selected
              </span>
              <button
                onClick={() => handleBatchStatus('done')}
                className="px-2 py-0.5 brutal-border bg-brutal-green text-[9px] font-black hover:bg-green-300 transition-colors"
              >
                Mark Done
              </button>
              <button
                onClick={() => handleBatchStatus('cancelled')}
                className="px-2 py-0.5 brutal-border bg-gray-200 text-[9px] font-black hover:bg-gray-300 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleBatchDelete}
                className="px-2 py-0.5 brutal-border bg-brutal-pink text-white text-[9px] font-black hover:bg-red-500 transition-colors flex items-center gap-1"
              >
                <Trash2 size={10} /> Delete
              </button>
            </>
          )}
        </div>

        <button
          onClick={() => setShowCreateModal(true)}
          className="brutal-btn bg-brutal-pink text-white text-[10px] flex items-center gap-1"
        >
          <Plus size={12} /> New Task
        </button>
      </div>

      {/* Table header */}
      <div className="flex items-center gap-3 px-3 py-1.5 brutal-border bg-gray-100 text-[9px] font-black uppercase text-gray-500">
        <input
          type="checkbox"
          checked={selectedIds.size === tasks.length && tasks.length > 0}
          onChange={toggleSelectAll}
          className="brutal-border w-3.5 h-3.5 accent-brutal-pink shrink-0"
        />
        <button onClick={() => handleSort('status')} className="w-2.5 flex items-center gap-0.5">
          <SortIcon field="status" />
        </button>
        <button onClick={() => handleSort('priority')} className="flex items-center gap-0.5 w-8">
          Pri <SortIcon field="priority" />
        </button>
        <button onClick={() => handleSort('title')} className="flex-1 flex items-center gap-0.5 text-left">
          Title <SortIcon field="title" />
        </button>
        <span className="w-20 text-center">Assignee</span>
        <button onClick={() => handleSort('createdAt')} className="w-14 text-right flex items-center gap-0.5 justify-end">
          Date <SortIcon field="createdAt" />
        </button>
      </div>

      {/* Rows */}
      <div className="flex-1 overflow-y-auto space-y-0">
        {sortedTasks.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-gray-400">
            <CheckSquare size={48} strokeWidth={1} className="mb-2 opacity-20" />
            <span className="text-sm italic">No tasks found</span>
          </div>
        ) : (
          sortedTasks.map(task => (
            <TaskListRow
              key={task.id}
              task={task}
              agents={agents}
              onClick={() => onTaskClick(task.id)}
              selected={selectedIds.has(task.id)}
              onToggleSelect={() => toggleSelect(task.id)}
              isExecuting={executingTaskIds?.has(task.id)}
            />
          ))
        )}
      </div>

      {/* Create Modal */}
      <TaskCreateModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onSubmit={onCreateTask}
        agents={agents}
        channelId={channelId}
      />
    </div>
  );
};
