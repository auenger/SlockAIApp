/**
 * TaskBoard — Kanban board view for Tasks.
 *
 * Displays tasks grouped by status in columns. Supports
 * drag-and-drop between columns to update status via @dnd-kit.
 */

import React, { useState } from 'react';
import {
  DndContext,
  DragOverlay,
  closestCorners,
  PointerSensor,
  useSensor,
  useSensors,
  type DragStartEvent,
  type DragEndEvent,
  type DragOverEvent,
} from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy, useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Plus } from 'lucide-react';
import { cn } from '../../lib/utils';
import { TaskCard } from './TaskCard';
import { STATUS_CONFIG } from './TaskStatusBadge';
import { TaskCreateModal, type TaskFormData } from './TaskCreateModal';
import type { Task, TaskStatus, AgentWithRuntime } from '../../types';

// ---------------------------------------------------------------------------
// Column definition
// ---------------------------------------------------------------------------

const COLUMNS: TaskStatus[] = ['todo', 'in_progress', 'in_review', 'done', 'blocked', 'cancelled'];

// ---------------------------------------------------------------------------
// Sortable Task Card wrapper
// ---------------------------------------------------------------------------

interface SortableTaskCardProps {
  task: Task;
  agents: AgentWithRuntime[];
  onClick: (taskId: string) => void;
}

function SortableTaskCard({ task, agents, onClick }: SortableTaskCardProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: task.id, data: { status: task.status } });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes}>
      <TaskCard
        task={task}
        agents={agents}
        onClick={() => onClick(task.id)}
        dragHandleProps={listeners}
        isDragging={isDragging}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// KanbanColumn
// ---------------------------------------------------------------------------

interface KanbanColumnProps {
  status: TaskStatus;
  tasks: Task[];
  agents: AgentWithRuntime[];
  onTaskClick: (taskId: string) => void;
  onCreateInColumn: (status: TaskStatus) => void;
}

function KanbanColumn({ status, tasks, agents, onTaskClick, onCreateInColumn }: KanbanColumnProps) {
  const config = STATUS_CONFIG[status];

  return (
    <div className="flex flex-col min-w-[200px] flex-1">
      {/* Column header */}
      <div className={cn('flex items-center justify-between px-3 py-2 brutal-border-b brutal-border-l brutal-border-r', config.bg)}>
        <span className="font-black text-xs uppercase">{config.label}</span>
        <span className="text-[10px] font-bold bg-white/50 px-1.5 brutal-border">{tasks.length}</span>
      </div>

      {/* Column body — droppable area */}
      <div className="flex-1 brutal-border-l brutal-border-r brutal-border-b bg-gray-50/50 p-2 space-y-2 min-h-[200px]">
        <SortableContext items={tasks.map(t => t.id)} strategy={verticalListSortingStrategy}>
          {tasks.map(task => (
            <SortableTaskCard
              key={task.id}
              task={task}
              agents={agents}
              onClick={onTaskClick}
            />
          ))}
        </SortableContext>

        {tasks.length === 0 && (
          <div className="text-center py-6 text-[10px] text-gray-400 italic">
            No tasks
          </div>
        )}

        {/* Add button (only for todo column) */}
        {status === 'todo' && (
          <button
            onClick={() => onCreateInColumn(status)}
            className="w-full py-1.5 brutal-border border-dashed text-[10px] font-bold text-gray-400 hover:text-black hover:bg-white hover:border-solid transition-all flex items-center justify-center gap-1"
          >
            <Plus size={10} /> Add Task
          </button>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// TaskBoard Props
// ---------------------------------------------------------------------------

interface TaskBoardProps {
  /** All tasks */
  tasks: Task[];
  /** Available agents */
  agents: AgentWithRuntime[];
  /** Callback when task status changes via drag */
  onStatusChange: (taskId: string, newStatus: TaskStatus) => Promise<void>;
  /** Callback when a task is clicked */
  onTaskClick: (taskId: string) => void;
  /** Callback to create a new task */
  onCreateTask: (data: TaskFormData) => Promise<void>;
  /** Channel ID for context */
  channelId?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskBoard: React.FC<TaskBoardProps> = ({
  tasks,
  agents,
  onStatusChange,
  onTaskClick,
  onCreateTask,
  channelId,
}) => {
  const [activeTask, setActiveTask] = useState<Task | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 8 },
    })
  );

  // Group tasks by status
  const tasksByStatus = COLUMNS.reduce<Record<TaskStatus, Task[]>>((acc, status) => {
    acc[status] = tasks.filter(t => t.status === status);
    return acc;
  }, {} as Record<TaskStatus, Task[]>);

  const handleDragStart = (event: DragStartEvent) => {
    const task = tasks.find(t => t.id === event.active.id);
    if (task) setActiveTask(task);
  };

  const handleDragOver = (_event: DragOverEvent) => {
    // Visual feedback could be added here
  };

  const handleDragEnd = async (event: DragEndEvent) => {
    setActiveTask(null);

    const { active, over } = event;
    if (!over) return;

    const taskId = String(active.id);

    // Determine target column by finding the closest column container
    // The over.id might be a task card or the column itself
    let targetStatus: TaskStatus | null = null;

    // Check if over.id is a status column
    if (COLUMNS.includes(over.id as TaskStatus)) {
      targetStatus = over.id as TaskStatus;
    } else {
      // over.id is a task — find which column it's in
      const overTask = tasks.find(t => t.id === over.id);
      if (overTask) {
        targetStatus = overTask.status;
      }
    }

    if (!targetStatus) return;

    const draggedTask = tasks.find(t => t.id === taskId);
    if (!draggedTask || draggedTask.status === targetStatus) return;

    await onStatusChange(taskId, targetStatus);
  };

  return (
    <div className="h-full flex flex-col">
      {/* Board header with create button */}
      <div className="flex items-center justify-between mb-3">
        <div />
        <button
          onClick={() => setShowCreateModal(true)}
          className="brutal-btn bg-brutal-pink text-white text-[10px] flex items-center gap-1"
        >
          <Plus size={12} /> New Task
        </button>
      </div>

      {/* Kanban columns */}
      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragStart={handleDragStart}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
      >
        <div className="flex-1 flex gap-0 overflow-x-auto">
          {COLUMNS.map(status => (
            <KanbanColumn
              key={status}
              status={status}
              tasks={tasksByStatus[status]}
              agents={agents}
              onTaskClick={onTaskClick}
              onCreateInColumn={() => setShowCreateModal(true)}
            />
          ))}
        </div>

        <DragOverlay>
          {activeTask && (
            <TaskCard
              task={activeTask}
              agents={agents}
              isDragging
              className="brutal-shadow rotate-3"
            />
          )}
        </DragOverlay>
      </DndContext>

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
