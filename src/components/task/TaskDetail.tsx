/**
 * TaskDetail — Side drawer showing full Task details.
 *
 * Displays all task fields, sub-tasks list, dependency management,
 * history timeline, and action buttons (execute, edit, delete, cancel).
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  X,
  Pencil,
  Trash2,
  Play,
  Square,
  Clock,
  Hash,
  User,
  Bot,
  Zap,
  Calendar,
  Link2,
  Plus,
  Minus,
  ChevronRight,
  AlertTriangle,
  GitBranch,
} from 'lucide-react';
import { cn } from '../../lib/utils';
import { AgentIcon } from '../AgentIcon';
import { TaskStatusBadge, TaskPriorityBadge } from './TaskStatusBadge';
import { TaskCreateModal, type TaskFormData } from './TaskCreateModal';
import {
  getTaskHistory,
  getTaskDependencies,
  getDependentTasks,
  getChildTasks,
  addTaskDependency,
  removeTaskDependency,
  getTask,
} from '../../lib/ipc';
import type {
  Task,
  TaskStatus,
  TaskHistoryEntry,
  TaskDependency,
  AgentWithRuntime,
} from '../../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TaskDetailProps {
  /** Whether the detail panel is open */
  isOpen: boolean;
  /** Close callback */
  onClose: () => void;
  /** Task to display */
  task: Task | null;
  /** Available agents */
  agents: AgentWithRuntime[];
  /** All tasks (for dependency picker) */
  allTasks?: Task[];
  /** Edit callback */
  onEdit: (taskId: string, data: TaskFormData) => Promise<void>;
  /** Delete callback */
  onDelete: (taskId: string) => Promise<void>;
  /** Status change callback */
  onStatusChange: (taskId: string, status: TaskStatus) => Promise<void>;
  /** Execute callback */
  onExecute?: (taskId: string) => Promise<void>;
  /** Cancel execution callback */
  onCancelExecution?: (taskId: string) => Promise<void>;
  /** Whether the task is currently being executed */
  isExecuting?: boolean;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskDetail: React.FC<TaskDetailProps> = ({
  isOpen,
  onClose,
  task,
  agents,
  allTasks,
  onEdit,
  onDelete,
  onStatusChange: _onStatusChange,
  onExecute,
  onCancelExecution,
  isExecuting = false,
}) => {
  const [showEditModal, setShowEditModal] = useState(false);
  const [history, setHistory] = useState<TaskHistoryEntry[]>([]);
  const [deleteConfirm, setDeleteConfirm] = useState(false);

  // Advanced state
  const [childTasks, setChildTasks] = useState<Task[]>([]);
  const [dependencies, setDependencies] = useState<TaskDependency[]>([]);
  const [dependents, setDependents] = useState<TaskDependency[]>([]);
  const [showAddDependency, setShowAddDependency] = useState(false);
  const [addDepTargetId, setAddDepTargetId] = useState('');
  const [depError, setDepError] = useState<string | null>(null);

  // Load data when task changes
  useEffect(() => {
    if (task?.id) {
      getTaskHistory(task.id).then(setHistory).catch(() => setHistory([]));
      getChildTasks(task.id).then(setChildTasks).catch(() => setChildTasks([]));
      getTaskDependencies(task.id).then(setDependencies).catch(() => setDependencies([]));
      getDependentTasks(task.id).then(setDependents).catch(() => setDependents([]));
      setDepError(null);
      setShowAddDependency(false);
    } else {
      setHistory([]);
      setChildTasks([]);
      setDependencies([]);
      setDependents([]);
    }
  }, [task?.id]);

  // --- Dependency management (all hooks must be called before any early return) ---

  const handleAddDependency = useCallback(async () => {
    if (!addDepTargetId || !task) return;
    setDepError(null);
    try {
      await addTaskDependency(task.id, addDepTargetId);
      // Refresh dependencies
      const newDeps = await getTaskDependencies(task.id);
      setDependencies(newDeps);
      // Refresh the task (status may have changed to blocked)
      const updated = await getTask(task.id);
      // Trigger parent refresh by calling onStatusChange with the updated status
      _onStatusChange(task.id, updated.status as TaskStatus);
      setShowAddDependency(false);
      setAddDepTargetId('');
    } catch (err) {
      setDepError(err instanceof Error ? err.message : String(err));
    }
  }, [task, addDepTargetId, _onStatusChange]);

  const handleRemoveDependency = useCallback(async (dependsOnId: string) => {
    if (!task) return;
    try {
      await removeTaskDependency(task.id, dependsOnId);
      const newDeps = await getTaskDependencies(task.id);
      setDependencies(newDeps);
    } catch (err) {
      console.error('Failed to remove dependency:', err);
    }
  }, [task]);

  // --- Early return AFTER all hooks ---

  if (!isOpen || !task) return null;

  const assignee = agents.find(a => a.agent.agent_id === task.assigneeId);

  const handleDelete = async () => {
    await onDelete(task.id);
    onClose();
  };

  const handleEdit = async (data: TaskFormData) => {
    await onEdit(task.id, data);
  };

  const canExecute = onExecute && !isExecuting &&
    (task.status === 'todo' || task.status === 'blocked') &&
    task.assigneeId;

  const canCancel = onCancelExecution && isExecuting;

  // Filter available tasks for dependency picker (exclude self and already-depended-on)
  const availableForDependency = (allTasks ?? []).filter(
    t => t.id !== task.id && !dependencies.some(d => d.dependsOnId === t.id)
  );

  // Resolve dependency/dependent task names
  const resolveTaskName = (taskId: string): string => {
    const found = (allTasks ?? []).find(t => t.id === taskId);
    return found ? found.title : taskId.slice(0, 8) + '...';
  };

  return (
    <>
      {/* Overlay */}
      <div
        className="fixed inset-0 bg-black/10 z-40"
        onClick={onClose}
      />

      {/* Panel */}
      <div className="fixed right-0 top-0 bottom-0 w-[400px] bg-white brutal-border-l brutal-shadow z-50 flex flex-col animate-slide-in-right">
        {/* Header */}
        <div className="flex items-center justify-between p-4 brutal-border-b bg-black text-white">
          <span className="font-black text-xs uppercase">Task Detail</span>
          <button onClick={onClose} className="p-1 hover:bg-white/20 transition-colors">
            <X size={16} />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {/* Title + Status */}
          <div>
            <div className="flex items-start gap-2 mb-2">
              <h2 className="font-black text-lg flex-1 leading-tight">{task.title}</h2>
              <TaskStatusBadge status={task.status} />
            </div>
            {/* Active execution indicator */}
            {isExecuting && (
              <div className="flex items-center gap-1.5 mb-2 p-1.5 bg-brutal-cyan/20 brutal-border text-xs">
                <span className="inline-block w-1.5 h-1.5 bg-brutal-cyan rounded-full animate-pulse" />
                <span className="font-bold text-brutal-cyan">Executing...</span>
                {task.executionMode === 'realtime' && (
                  <span className="text-gray-500 text-[10px]">in channel</span>
                )}
              </div>
            )}
            {task.description && (
              <p className="text-xs text-gray-600 leading-relaxed whitespace-pre-wrap">
                {task.description}
              </p>
            )}
          </div>

          {/* Fields */}
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-xs">
              <Zap size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Priority</span>
              <TaskPriorityBadge priority={task.priority} />
            </div>

            <div className="flex items-center gap-2 text-xs">
              <Bot size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Assignee</span>
              {assignee ? (
                <div className="flex items-center gap-1.5">
                  <AgentIcon
                    icon={assignee.agent.icon}
                    emoji={assignee.agent.emoji}
                    size="sm"
                    bgColor="bg-brutal-cyan"
                  />
                  <span className="font-bold">{assignee.agent.name}</span>
                </div>
              ) : (
                <span className="text-gray-400 italic">Unassigned</span>
              )}
            </div>

            <div className="flex items-center gap-2 text-xs">
              <User size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Creator</span>
              <span className="font-bold">
                {task.creatorType === 'user' ? 'You' : task.creatorId}
              </span>
            </div>

            {task.parentTaskId && (
              <div className="flex items-center gap-2 text-xs">
                <GitBranch size={12} className="text-gray-400" />
                <span className="text-gray-500 font-bold w-20">Parent</span>
                <span className="font-mono text-[10px]">{resolveTaskName(task.parentTaskId)}</span>
              </div>
            )}

            {task.channelId && (
              <div className="flex items-center gap-2 text-xs">
                <Hash size={12} className="text-gray-400" />
                <span className="text-gray-500 font-bold w-20">Channel</span>
                <span className="font-mono text-[10px]">{task.channelId}</span>
              </div>
            )}

            <div className="flex items-center gap-2 text-xs">
              <Play size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Mode</span>
              <span className={cn(
                'px-1.5 py-0.5 brutal-border text-[9px] font-black',
                task.executionMode === 'realtime' ? 'bg-brutal-cyan' : 'bg-purple-400 text-white'
              )}>
                {task.executionMode === 'realtime' ? 'REALTIME' : 'ASYNC'}
              </span>
            </div>

            <div className="flex items-center gap-2 text-xs">
              <Link2 size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Source</span>
              <span className="text-[10px] font-mono text-gray-600 uppercase">{task.source}</span>
            </div>

            <div className="flex items-center gap-2 text-xs">
              <Calendar size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Created</span>
              <span className="text-[10px] font-mono">
                {new Date(task.createdAt).toLocaleString()}
              </span>
            </div>
            <div className="flex items-center gap-2 text-xs">
              <Clock size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Updated</span>
              <span className="text-[10px] font-mono">
                {new Date(task.updatedAt).toLocaleString()}
              </span>
            </div>
            {task.completedAt && (
              <div className="flex items-center gap-2 text-xs">
                <Clock size={12} className="text-brutal-green" />
                <span className="text-gray-500 font-bold w-20">Completed</span>
                <span className="text-[10px] font-mono">
                  {new Date(task.completedAt).toLocaleString()}
                </span>
              </div>
            )}
          </div>

          {/* Result */}
          {task.result && (task.status === 'in_review' || task.status === 'done') && (
            <div>
              <div className="text-[10px] font-black uppercase text-gray-500 mb-1">Result</div>
              <div className={cn(
                'p-2 brutal-border text-xs leading-relaxed',
                task.result.startsWith('FAILED:') ? 'bg-red-50 text-red-700' : 'bg-brutal-bg'
              )}>
                {task.result.startsWith('FAILED:') ? task.result.slice(7) : task.result}
              </div>
            </div>
          )}

          {/* Sub Tasks */}
          {task.childTaskCount > 0 && (
            <div>
              <div className="text-[10px] font-black uppercase text-gray-500 mb-1">
                Sub Tasks ({childTasks.length})
              </div>
              <div className="space-y-1">
                {childTasks.map(child => (
                  <div
                    key={child.id}
                    className="flex items-center gap-2 p-1.5 brutal-border text-xs bg-white hover:bg-gray-50 transition-colors"
                  >
                    <TaskStatusBadge status={child.status} />
                    <span className="font-bold flex-1 truncate">{child.title}</span>
                    {child.assigneeId && (
                      <span className="text-[9px] text-gray-400">
                        @{agents.find(a => a.agent.agent_id === child.assigneeId)?.agent.name ?? child.assigneeId}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Dependencies */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <div className="text-[10px] font-black uppercase text-gray-500">
                Dependencies ({dependencies.length})
              </div>
              <button
                onClick={() => setShowAddDependency(!showAddDependency)}
                className="p-0.5 brutal-border hover:bg-gray-100 transition-colors"
                title="Add dependency"
              >
                {showAddDependency ? <Minus size={10} /> : <Plus size={10} />}
              </button>
            </div>

            {depError && (
              <div className="flex items-center gap-1 p-1.5 bg-red-50 text-red-600 text-[10px] mb-1 brutal-border">
                <AlertTriangle size={10} />
                {depError}
              </div>
            )}

            {showAddDependency && (
              <div className="flex items-center gap-1 mb-1">
                <select
                  value={addDepTargetId}
                  onChange={e => setAddDepTargetId(e.target.value)}
                  className="flex-1 brutal-border bg-white px-1.5 py-1 text-[10px] focus:outline-none"
                >
                  <option value="">Select task...</option>
                  {availableForDependency.map(t => (
                    <option key={t.id} value={t.id}>{t.title}</option>
                  ))}
                </select>
                <button
                  onClick={handleAddDependency}
                  disabled={!addDepTargetId}
                  className={cn(
                    'px-2 py-1 brutal-border text-[10px] font-black',
                    addDepTargetId
                      ? 'bg-brutal-cyan hover:bg-brutal-cyan/80'
                      : 'bg-gray-100 text-gray-400 cursor-not-allowed'
                  )}
                >
                  Add
                </button>
              </div>
            )}

            {dependencies.length > 0 ? (
              <div className="space-y-1">
                {dependencies.map(dep => (
                  <div
                    key={dep.dependsOnId}
                    className="flex items-center gap-2 p-1.5 brutal-border text-xs bg-white"
                  >
                    <ChevronRight size={10} className="text-gray-400" />
                    <span className="font-bold flex-1 truncate">
                      {resolveTaskName(dep.dependsOnId)}
                    </span>
                    <button
                      onClick={() => handleRemoveDependency(dep.dependsOnId)}
                      className="p-0.5 hover:bg-red-100 transition-colors"
                      title="Remove dependency"
                    >
                      <X size={10} className="text-gray-400 hover:text-red-500" />
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-xs text-gray-400 italic">No dependencies</div>
            )}

            {/* Tasks that depend on this one (reverse deps) */}
            {dependents.length > 0 && (
              <div className="mt-2">
                <div className="text-[10px] font-black uppercase text-gray-400 mb-1">
                  Blocks ({dependents.length})
                </div>
                <div className="space-y-1">
                  {dependents.map(dep => (
                    <div
                      key={dep.taskId}
                      className="flex items-center gap-2 p-1.5 brutal-border text-xs bg-gray-50"
                    >
                      <span className="text-gray-400 text-[10px]">blocks</span>
                      <span className="font-bold flex-1 truncate">
                        {resolveTaskName(dep.taskId)}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* History Timeline */}
          {history.length > 0 && (
            <div>
              <div className="text-[10px] font-black uppercase text-gray-500 mb-2">
                Activity
              </div>
              <div className="space-y-1">
                {history.slice(0, 20).map((entry) => (
                  <div
                    key={entry.id}
                    className="flex items-start gap-2 text-[10px] py-1 brutal-border border-transparent border-l-2 hover:border-l-black pl-2 transition-colors"
                  >
                    <span className="text-gray-400 font-mono shrink-0 w-24">
                      {new Date(entry.changedAt).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </span>
                    <span className="text-gray-600">
                      <span className="font-bold">{entry.field}</span>
                      {entry.oldValue && entry.newValue ? (
                        <>: {entry.oldValue} {'->'} {entry.newValue}</>
                      ) : entry.newValue ? (
                        <>: {entry.newValue}</>
                      ) : null}
                    </span>
                    <span className="text-gray-400 ml-auto shrink-0 truncate max-w-[80px]" title={entry.changedBy}>
                      {entry.changedBy}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Footer Actions */}
        <div className="p-4 brutal-border-t bg-gray-50 space-y-2">
          {/* Execution controls */}
          <div className="flex gap-2">
            {canExecute && (
              <button
                onClick={() => onExecute(task.id)}
                className="brutal-btn bg-brutal-cyan text-black text-[10px] flex-1 flex items-center justify-center gap-1"
              >
                <Play size={12} /> Execute
              </button>
            )}
            {canCancel && (
              <button
                onClick={() => onCancelExecution(task.id)}
                className="brutal-btn bg-red-400 text-white text-[10px] flex-1 flex items-center justify-center gap-1"
              >
                <Square size={12} /> Cancel Execution
              </button>
            )}
          </div>

          {/* Edit / Delete */}
          <div className="flex gap-2">
            <button
              onClick={() => setShowEditModal(true)}
              className="brutal-btn bg-white text-[10px] flex-1 flex items-center justify-center gap-1"
            >
              <Pencil size={12} /> Edit
            </button>
            {deleteConfirm ? (
              <div className="flex gap-1 flex-1">
                <button
                  onClick={handleDelete}
                  className="brutal-btn bg-brutal-pink text-white text-[10px] flex-1"
                >
                  Confirm
                </button>
                <button
                  onClick={() => setDeleteConfirm(false)}
                  className="brutal-btn bg-gray-200 text-[10px]"
                >
                  No
                </button>
              </div>
            ) : (
              <button
                onClick={() => setDeleteConfirm(true)}
                className="brutal-btn bg-gray-200 text-[10px] flex-1 flex items-center justify-center gap-1 hover:bg-red-100"
              >
                <Trash2 size={12} /> Delete
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Edit Modal */}
      <TaskCreateModal
        isOpen={showEditModal}
        onClose={() => setShowEditModal(false)}
        onSubmit={handleEdit}
        agents={agents}
        task={task}
        channelId={task.channelId ?? undefined}
      />
    </>
  );
};
