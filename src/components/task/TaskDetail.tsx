/**
 * TaskDetail — Side drawer showing full Task details.
 *
 * Displays all task fields, history timeline, and action buttons
 * (execute, edit, delete, cancel).
 */

import React, { useState, useEffect } from 'react';
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
} from 'lucide-react';
import { cn } from '../../lib/utils';
import { AgentIcon } from '../AgentIcon';
import { TaskStatusBadge, TaskPriorityBadge } from './TaskStatusBadge';
import { TaskCreateModal, type TaskFormData } from './TaskCreateModal';
import { getTaskHistory } from '../../lib/ipc';
import type { Task, TaskStatus, TaskHistoryEntry, AgentWithRuntime } from '../../types';

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

  // Load history when task changes
  useEffect(() => {
    if (task?.id) {
      getTaskHistory(task.id)
        .then(setHistory)
        .catch(() => setHistory([]));
    } else {
      setHistory([]);
    }
  }, [task?.id]);

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
            {task.description && (
              <p className="text-xs text-gray-600 leading-relaxed whitespace-pre-wrap">
                {task.description}
              </p>
            )}
          </div>

          {/* Fields */}
          <div className="space-y-2">
            {/* Priority */}
            <div className="flex items-center gap-2 text-xs">
              <Zap size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Priority</span>
              <TaskPriorityBadge priority={task.priority} />
            </div>

            {/* Assignee */}
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

            {/* Creator */}
            <div className="flex items-center gap-2 text-xs">
              <User size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Creator</span>
              <span className="font-bold">
                {task.creatorType === 'user' ? 'You' : task.creatorId}
              </span>
            </div>

            {/* Channel */}
            {task.channelId && (
              <div className="flex items-center gap-2 text-xs">
                <Hash size={12} className="text-gray-400" />
                <span className="text-gray-500 font-bold w-20">Channel</span>
                <span className="font-mono text-[10px]">{task.channelId}</span>
              </div>
            )}

            {/* Execution Mode */}
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

            {/* Source */}
            <div className="flex items-center gap-2 text-xs">
              <Link2 size={12} className="text-gray-400" />
              <span className="text-gray-500 font-bold w-20">Source</span>
              <span className="text-[10px] font-mono text-gray-600 uppercase">{task.source}</span>
            </div>

            {/* Dates */}
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
                Sub Tasks ({task.childTaskCount})
              </div>
              <div className="text-xs text-gray-400 italic">
                Expand to view sub-tasks
              </div>
            </div>
          )}

          {/* History Timeline */}
          {history.length > 0 && (
            <div>
              <div className="text-[10px] font-black uppercase text-gray-500 mb-2">
                Activity
              </div>
              <div className="space-y-1">
                {history.slice(0, 10).map((entry) => (
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
                    <span className="text-gray-400 ml-auto shrink-0">{entry.changedBy}</span>
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
