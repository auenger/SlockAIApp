/**
 * TaskStatusBadge — Status indicator for Task items.
 *
 * Renders a colored badge with the task status label,
 * using the brutalist design system.
 */

import React from 'react';
import { cn } from '../../lib/utils';
import type { TaskStatus } from '../../types';

// ---------------------------------------------------------------------------
// Status config
// ---------------------------------------------------------------------------

interface StatusConfig {
  label: string;
  bg: string;
  text: string;
}

const STATUS_CONFIG: Record<TaskStatus, StatusConfig> = {
  todo: { label: 'Todo', bg: 'bg-brutal-yellow', text: 'text-black' },
  in_progress: { label: 'In Progress', bg: 'bg-brutal-cyan', text: 'text-black' },
  in_review: { label: 'In Review', bg: 'bg-purple-400', text: 'text-white' },
  done: { label: 'Done', bg: 'bg-brutal-green', text: 'text-black' },
  blocked: { label: 'Blocked', bg: 'bg-red-400', text: 'text-white' },
  cancelled: { label: 'Cancelled', bg: 'bg-gray-300', text: 'text-gray-700' },
};

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TaskStatusBadgeProps {
  status: TaskStatus;
  /** Compact mode — no text, just colored dot */
  compact?: boolean;
  /** Additional CSS classes */
  className?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskStatusBadge: React.FC<TaskStatusBadgeProps> = ({
  status,
  compact = false,
  className,
}) => {
  const config = STATUS_CONFIG[status];

  if (compact) {
    return (
      <span
        className={cn(
          'inline-block w-2.5 h-2.5 brutal-border',
          config.bg,
          className
        )}
        title={config.label}
      />
    );
  }

  return (
    <span
      className={cn(
        'inline-flex items-center px-1.5 py-0.5 brutal-border text-[9px] font-black uppercase whitespace-nowrap',
        config.bg,
        config.text,
        className
      )}
    >
      {config.label}
    </span>
  );
};

// ---------------------------------------------------------------------------
// Priority badge
// ---------------------------------------------------------------------------

interface TaskPriorityBadgeProps {
  priority: number;
  className?: string;
}

const PRIORITY_CONFIG: Record<number, { label: string; bg: string }> = {
  1: { label: 'P1', bg: 'bg-red-400 text-white' },
  2: { label: 'P2', bg: 'bg-orange-400 text-white' },
  3: { label: 'P3', bg: 'bg-brutal-yellow text-black' },
  4: { label: 'P4', bg: 'bg-gray-200 text-gray-700' },
  5: { label: 'P5', bg: 'bg-gray-100 text-gray-500' },
};

export const TaskPriorityBadge: React.FC<TaskPriorityBadgeProps> = ({
  priority,
  className,
}) => {
  const config = PRIORITY_CONFIG[priority] || PRIORITY_CONFIG[3];

  return (
    <span
      className={cn(
        'inline-flex items-center px-1 py-0.5 brutal-border text-[8px] font-black',
        config.bg,
        className
      )}
    >
      {config.label}
    </span>
  );
};

/** Get the status config for external use */
export { STATUS_CONFIG };
