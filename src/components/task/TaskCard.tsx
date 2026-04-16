/**
 * TaskCard — Compact card for a Task item.
 *
 * Used in both the Kanban board columns and the list view rows.
 * Displays status, title, priority, assignee, and channel.
 * Shows async execution animation when task is actively executing.
 */

import React from 'react';
import { cn } from '../../lib/utils';
import { AgentIcon } from '../AgentIcon';
import { TaskStatusBadge, TaskPriorityBadge } from './TaskStatusBadge';
import type { Task, AgentWithRuntime } from '../../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TaskCardProps {
  /** Task data */
  task: Task;
  /** Available agents for resolving assignee info */
  agents: AgentWithRuntime[];
  /** Click handler — opens detail */
  onClick?: () => void;
  /** Drag handle props (from @dnd-kit) */
  dragHandleProps?: Record<string, unknown>;
  /** Whether this card is being dragged */
  isDragging?: boolean;
  /** Whether this task is currently being executed (async) */
  isExecuting?: boolean;
  /** Additional CSS classes */
  className?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskCard: React.FC<TaskCardProps> = ({
  task,
  agents,
  onClick,
  dragHandleProps,
  isDragging = false,
  isExecuting = false,
  className,
}) => {
  const assignee = agents.find(a => a.agent.agent_id === task.assigneeId);

  return (
    <div
      {...dragHandleProps}
      onClick={onClick}
      className={cn(
        'brutal-border bg-white p-2.5 cursor-pointer transition-all hover:translate-x-[-1px] hover:translate-y-[-1px] hover:brutal-shadow-sm',
        isDragging && 'opacity-70 brutal-shadow rotate-2',
        isExecuting && 'ring-2 ring-brutal-cyan/50 ring-offset-1',
        className
      )}
    >
      {/* Top row: priority + status + async indicator */}
      <div className="flex items-center gap-1.5 mb-1.5">
        <TaskPriorityBadge priority={task.priority} />
        <TaskStatusBadge status={task.status} />
        {isExecuting && task.executionMode === 'async' && (
          <span className="flex items-center gap-0.5 ml-auto">
            <span className="inline-block w-1.5 h-1.5 bg-brutal-cyan rounded-full animate-pulse" />
            <span className="text-[8px] font-black text-brutal-cyan uppercase">Async</span>
          </span>
        )}
      </div>

      {/* Title */}
      <div className="font-bold text-xs leading-snug mb-1.5 line-clamp-2">
        {task.title}
      </div>

      {/* Description preview */}
      {task.description && (
        <div className="text-[10px] text-gray-500 line-clamp-2 mb-2 leading-relaxed">
          {task.description}
        </div>
      )}

      {/* Bottom row: assignee + meta */}
      <div className="flex items-center gap-1.5">
        {assignee ? (
          <div className="flex items-center gap-1">
            <AgentIcon
              icon={assignee.agent.icon}
              emoji={assignee.agent.emoji}
              size="sm"
              bgColor="bg-brutal-cyan"
            />
            <span className="text-[10px] font-bold truncate max-w-[80px]">
              {assignee.agent.name}
            </span>
          </div>
        ) : (
          <span className="text-[9px] text-gray-400 italic">Unassigned</span>
        )}

        {/* Child task count */}
        {task.childTaskCount > 0 && (
          <span className="text-[8px] text-gray-400 ml-auto font-mono">
            {task.childTaskCount} sub{task.childTaskCount !== 1 ? 's' : ''}
          </span>
        )}
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// TaskListRow — compact row variant for list view
// ---------------------------------------------------------------------------

interface TaskListRowProps {
  task: Task;
  agents: AgentWithRuntime[];
  onClick?: () => void;
  selected?: boolean;
  onToggleSelect?: () => void;
  isExecuting?: boolean;
}

export const TaskListRow: React.FC<TaskListRowProps> = ({
  task,
  agents,
  onClick,
  selected = false,
  onToggleSelect,
  isExecuting = false,
}) => {
  const assignee = agents.find(a => a.agent.agent_id === task.assigneeId);

  return (
    <div
      className={cn(
        'flex items-center gap-3 px-3 py-2 brutal-border bg-white hover:bg-gray-50 transition-colors cursor-pointer',
        selected && 'bg-brutal-yellow/20 border-l-4 border-l-brutal-pink',
        isExecuting && 'ring-2 ring-brutal-cyan/50 ring-offset-1'
      )}
      onClick={onClick}
    >
      {/* Checkbox */}
      <input
        type="checkbox"
        checked={selected}
        onChange={(e) => {
          e.stopPropagation();
          onToggleSelect?.();
        }}
        onClick={(e) => e.stopPropagation()}
        className="brutal-border w-3.5 h-3.5 accent-brutal-pink shrink-0"
      />

      {/* Status */}
      <TaskStatusBadge status={task.status} compact />

      {/* Priority */}
      <TaskPriorityBadge priority={task.priority} />

      {/* Async execution indicator */}
      {isExecuting && task.executionMode === 'async' && (
        <span className="flex items-center gap-0.5">
          <span className="inline-block w-1.5 h-1.5 bg-brutal-cyan rounded-full animate-pulse" />
        </span>
      )}

      {/* Title */}
      <span className="font-bold text-xs flex-1 truncate">{task.title}</span>

      {/* Assignee */}
      {assignee ? (
        <div className="flex items-center gap-1 shrink-0">
          <AgentIcon
            icon={assignee.agent.icon}
            emoji={assignee.agent.emoji}
            size="sm"
            bgColor="bg-brutal-cyan"
          />
          <span className="text-[10px] font-bold max-w-[60px] truncate">{assignee.agent.name}</span>
        </div>
      ) : (
        <span className="text-[9px] text-gray-400 shrink-0 italic">Unassigned</span>
      )}

      {/* Date */}
      <span className="text-[9px] text-gray-400 font-mono shrink-0">
        {new Date(task.createdAt).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}
      </span>
    </div>
  );
};
