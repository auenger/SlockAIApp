/**
 * Task Execution UI Components.
 *
 * Provides buttons, progress indicators, and result display
 * for task execution lifecycle.
 */

import { useCallback } from 'react';
import { cn } from '../lib/utils';
import type { Task, TaskStatus } from '../types';
import type { ActiveTaskInfo } from '../lib/useTaskEngine';

// ---------------------------------------------------------------------------
// TaskExecuteButton — Play button to start task execution
// ---------------------------------------------------------------------------

interface TaskExecuteButtonProps {
  task: Task;
  isActive: boolean;
  onExecute: (taskId: string) => Promise<boolean>;
  disabled?: boolean;
}

export function TaskExecuteButton({
  task,
  isActive,
  onExecute,
  disabled = false,
}: TaskExecuteButtonProps) {
  const canExecute =
    !disabled &&
    !isActive &&
    (task.status === 'todo' || task.status === 'blocked') &&
    !!task.assigneeId;

  const handleClick = useCallback(async () => {
    if (canExecute) {
      await onExecute(task.id);
    }
  }, [canExecute, onExecute, task.id]);

  return (
    <button
      onClick={handleClick}
      disabled={!canExecute}
      className={cn(
        'inline-flex items-center justify-center w-7 h-7 rounded-md transition-colors',
        canExecute
          ? 'text-green-400 hover:bg-green-400/20 hover:text-green-300 cursor-pointer'
          : 'text-zinc-600 cursor-not-allowed'
      )}
      title={
        isActive
          ? 'Task is executing...'
          : !task.assigneeId
            ? 'No agent assigned'
            : task.status === 'todo' || task.status === 'blocked'
              ? 'Execute task'
              : `Cannot execute (status: ${task.status})`
      }
    >
      {/* Play triangle */}
      <svg
        width="14"
        height="14"
        viewBox="0 0 14 14"
        fill="currentColor"
      >
        <path d="M3 1.5L12 7L3 12.5V1.5Z" />
      </svg>
    </button>
  );
}

// ---------------------------------------------------------------------------
// TaskCancelButton — Cancel a running task
// ---------------------------------------------------------------------------

interface TaskCancelButtonProps {
  taskId: string;
  onCancel: (taskId: string) => Promise<boolean>;
}

export function TaskCancelButton({ taskId, onCancel }: TaskCancelButtonProps) {
  const handleClick = useCallback(async () => {
    await onCancel(taskId);
  }, [onCancel, taskId]);

  return (
    <button
      onClick={handleClick}
      className="inline-flex items-center justify-center w-7 h-7 rounded-md text-red-400 hover:bg-red-400/20 hover:text-red-300 transition-colors cursor-pointer"
      title="Cancel execution"
    >
      {/* Stop square */}
      <svg
        width="12"
        height="12"
        viewBox="0 0 12 12"
        fill="currentColor"
      >
        <rect x="1" y="1" width="10" height="10" rx="1" />
      </svg>
    </button>
  );
}

// ---------------------------------------------------------------------------
// TaskProgressBar — Execution progress indicator
// ---------------------------------------------------------------------------

interface TaskProgressBarProps {
  activeTask: ActiveTaskInfo | undefined;
}

export function TaskProgressBar({ activeTask }: TaskProgressBarProps) {
  if (!activeTask) return null;

  return (
    <div className="flex items-center gap-2 mt-1.5">
      {/* Animated progress bar */}
      <div className="flex-1 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
        <div
          className={cn(
            'h-full rounded-full transition-all',
            activeTask.mode === 'realtime'
              ? 'bg-blue-500 animate-pulse'
              : 'bg-purple-500 animate-pulse'
          )}
          style={{ width: '60%' }}
        />
      </div>

      {/* Mode badge */}
      <span
        className={cn(
          'text-[10px] font-medium px-1.5 py-0.5 rounded-full',
          activeTask.mode === 'realtime'
            ? 'bg-blue-500/20 text-blue-400'
            : 'bg-purple-500/20 text-purple-400'
        )}
      >
        {activeTask.mode === 'realtime' ? 'LIVE' : 'ASYNC'}
      </span>

      {/* Progress text */}
      {activeTask.progress && (
        <span className="text-[10px] text-zinc-500 truncate max-w-[120px]">
          {activeTask.progress}
        </span>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// TaskResultDisplay — Show execution result
// ---------------------------------------------------------------------------

interface TaskResultDisplayProps {
  result: string | undefined;
  status: TaskStatus;
}

export function TaskResultDisplay({ result, status }: TaskResultDisplayProps) {
  if (!result || (status !== 'in_review' && status !== 'done' && status !== 'blocked')) {
    return null;
  }

  const isFailure = result.startsWith('FAILED:');

  return (
    <div
      className={cn(
        'mt-2 p-2 rounded-md text-xs border',
        isFailure
          ? 'bg-red-500/10 border-red-500/30 text-red-300'
          : 'bg-green-500/10 border-green-500/30 text-green-300'
      )}
    >
      <div className="flex items-center gap-1 mb-1">
        {isFailure ? (
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <path d="M6 1L11 10H1L6 1Z" stroke="currentColor" fill="none" strokeWidth="1.5" />
            <line x1="6" y1="4.5" x2="6" y2="7" stroke="currentColor" strokeWidth="1.2" />
            <circle cx="6" cy="8.5" r="0.6" fill="currentColor" />
          </svg>
        ) : (
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <path d="M2 6L5 9L10 3" stroke="currentColor" fill="none" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        )}
        <span className="font-medium">
          {isFailure ? 'Execution Failed' : 'Execution Result'}
        </span>
      </div>
      <p className="text-zinc-400 whitespace-pre-wrap break-words line-clamp-4">
        {isFailure ? result.slice(7) : result}
      </p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// TaskExecutionStatus — Combined status badge for task card
// ---------------------------------------------------------------------------

interface TaskExecutionStatusProps {
  task: Task;
  isActive: boolean;
  activeTask?: ActiveTaskInfo;
}

export function TaskExecutionStatus({
  task,
  isActive,
  activeTask,
}: TaskExecutionStatusProps) {
  if (isActive && activeTask) {
    return (
      <div className="flex items-center gap-1.5">
        {/* Spinning indicator */}
        <div className="w-3 h-3 relative">
          <div
            className={cn(
              'absolute inset-0 rounded-full border-2 border-t-transparent animate-spin',
              activeTask.mode === 'realtime'
                ? 'border-blue-400'
                : 'border-purple-400'
            )}
          />
        </div>
        <span className="text-xs text-zinc-400">
          {activeTask.mode === 'realtime' ? 'Executing...' : 'Running async...'}
        </span>
      </div>
    );
  }

  switch (task.status) {
    case 'in_progress':
      return (
        <span className="text-xs text-blue-400">In Progress</span>
      );
    case 'in_review':
      return (
        <span className="text-xs text-yellow-400">In Review</span>
      );
    case 'done':
      return (
        <span className="text-xs text-green-400">Done</span>
      );
    case 'blocked':
      return (
        <span className="text-xs text-red-400">
          {task.result?.startsWith('FAILED:') ? 'Failed' : 'Blocked'}
        </span>
      );
    case 'cancelled':
      return (
        <span className="text-xs text-zinc-500">Cancelled</span>
      );
    default:
      return (
        <span className="text-xs text-zinc-500 capitalize">{task.status}</span>
      );
  }
}

// ---------------------------------------------------------------------------
// TaskExecutionControls — Combined execute/cancel/progress controls
// ---------------------------------------------------------------------------

interface TaskExecutionControlsProps {
  task: Task;
  isActive: boolean;
  activeTask?: ActiveTaskInfo;
  onExecute: (taskId: string) => Promise<boolean>;
  onCancel: (taskId: string) => Promise<boolean>;
}

export function TaskExecutionControls({
  task,
  isActive,
  activeTask,
  onExecute,
  onCancel,
}: TaskExecutionControlsProps) {
  return (
    <div className="flex items-center gap-1">
      {isActive ? (
        <TaskCancelButton taskId={task.id} onCancel={onCancel} />
      ) : (
        <TaskExecuteButton task={task} isActive={false} onExecute={onExecute} />
      )}

      <TaskExecutionStatus
        task={task}
        isActive={isActive}
        activeTask={activeTask}
      />

      {isActive && activeTask && (
        <div className="ml-auto">
          <span className="text-[10px] text-zinc-600">
            {activeTask.agentId}
          </span>
        </div>
      )}
    </div>
  );
}
