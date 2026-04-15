/**
 * TaskCreateModal — Dialog for creating and editing Tasks.
 *
 * Supports all Task fields: title, description, priority,
 * assignee, execution mode, and channel binding.
 */

import React, { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { TaskAssignDropdown } from './TaskAssignDropdown';
import type { Task, TaskPriority, TaskExecutionMode, AgentWithRuntime } from '../../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TaskCreateModalProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Close callback */
  onClose: () => void;
  /** Submit callback — receives form data */
  onSubmit: (data: TaskFormData) => Promise<void>;
  /** Available agents for assignment */
  agents: AgentWithRuntime[];
  /** Existing task to edit (null = create mode) */
  task?: Task | null;
  /** Channel ID to pre-bind (from Channel context) */
  channelId?: string;
}

export interface TaskFormData {
  title: string;
  description: string;
  priority: TaskPriority;
  assigneeId: string | null;
  executionMode: TaskExecutionMode;
  channelId: string | null;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskCreateModal: React.FC<TaskCreateModalProps> = ({
  isOpen,
  onClose,
  onSubmit,
  agents,
  task = null,
  channelId,
}) => {
  const isEditing = !!task;

  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<TaskPriority>(3);
  const [assigneeId, setAssigneeId] = useState<string | null>(null);
  const [executionMode, setExecutionMode] = useState<TaskExecutionMode>('realtime');
  const [selectedChannelId, setSelectedChannelId] = useState<string | null>(channelId ?? null);
  const [submitting, setSubmitting] = useState(false);

  // Populate form when editing
  useEffect(() => {
    if (task) {
      setTitle(task.title);
      setDescription(task.description);
      setPriority(task.priority);
      setAssigneeId(task.assigneeId ?? null);
      setExecutionMode(task.executionMode);
      setSelectedChannelId(task.channelId ?? null);
    } else {
      setTitle('');
      setDescription('');
      setPriority(3);
      setAssigneeId(null);
      setExecutionMode('realtime');
      setSelectedChannelId(channelId ?? null);
    }
  }, [task, channelId, isOpen]);

  const handleSubmit = async () => {
    if (!title.trim()) return;
    setSubmitting(true);
    try {
      await onSubmit({
        title: title.trim(),
        description: description.trim(),
        priority,
        assigneeId,
        executionMode,
        channelId: selectedChannelId,
      });
      onClose();
    } finally {
      setSubmitting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50" onClick={onClose}>
      <div
        className="bg-white brutal-border brutal-shadow w-[480px] max-h-[90vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 brutal-border-b bg-black text-white">
          <span className="font-black text-sm uppercase">
            {isEditing ? 'Edit Task' : 'New Task'}
          </span>
          <button onClick={onClose} className="p-1 hover:bg-white/20 transition-colors">
            <X size={16} />
          </button>
        </div>

        {/* Form */}
        <div className="p-4 space-y-4">
          {/* Title */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Title <span className="text-brutal-pink">*</span>
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="What needs to be done?"
              className="w-full brutal-border bg-white px-3 py-2 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter' && title.trim()) handleSubmit();
              }}
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Add more details..."
              rows={3}
              className="w-full brutal-border bg-white px-3 py-2 text-xs focus:outline-none focus:bg-brutal-bg resize-none"
            />
          </div>

          {/* Priority */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Priority
            </label>
            <div className="flex gap-1">
              {([1, 2, 3, 4, 5] as TaskPriority[]).map((p) => (
                <button
                  key={p}
                  type="button"
                  onClick={() => setPriority(p)}
                  className={cn(
                    'px-3 py-1 brutal-border text-[10px] font-black transition-all',
                    priority === p
                      ? p <= 1 ? 'bg-red-400 text-white'
                        : p <= 2 ? 'bg-orange-400 text-white'
                        : p <= 3 ? 'bg-brutal-yellow text-black'
                        : 'bg-gray-200 text-gray-700'
                      : 'bg-white hover:bg-gray-100'
                  )}
                >
                  P{p}
                </button>
              ))}
            </div>
          </div>

          {/* Assignee */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Assignee
            </label>
            <TaskAssignDropdown
              agents={agents}
              value={assigneeId}
              onChange={setAssigneeId}
            />
          </div>

          {/* Execution Mode */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Execution Mode
            </label>
            <div className="flex gap-1">
              {(['realtime', 'async'] as TaskExecutionMode[]).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  onClick={() => setExecutionMode(mode)}
                  className={cn(
                    'px-3 py-1 brutal-border text-[10px] font-black uppercase transition-all',
                    executionMode === mode
                      ? mode === 'realtime' ? 'bg-brutal-cyan text-black' : 'bg-purple-400 text-white'
                      : 'bg-white hover:bg-gray-100'
                  )}
                >
                  {mode === 'realtime' ? 'Realtime' : 'Async'}
                </button>
              ))}
            </div>
          </div>

          {/* Channel (optional) */}
          {!channelId && (
            <div>
              <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
                Channel (optional)
              </label>
              <input
                type="text"
                value={selectedChannelId ?? ''}
                onChange={(e) => setSelectedChannelId(e.target.value || null)}
                placeholder="Channel ID"
                className="w-full brutal-border bg-white px-3 py-1.5 text-xs font-mono focus:outline-none focus:bg-brutal-bg"
              />
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 p-4 brutal-border-t bg-gray-50">
          <button
            onClick={onClose}
            className="px-4 py-1.5 brutal-border bg-gray-200 text-xs font-black hover:bg-gray-300 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!title.trim() || submitting}
            className={cn(
              'brutal-btn text-xs font-black',
              title.trim() && !submitting
                ? 'bg-brutal-pink text-white'
                : 'bg-gray-200 text-gray-400 cursor-not-allowed shadow-none'
            )}
          >
            {submitting ? 'Saving...' : isEditing ? 'Save Changes' : 'Create Task'}
          </button>
        </div>
      </div>
    </div>
  );
};
