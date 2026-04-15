/**
 * TaskSuggestionCard — Interactive message card for task suggestions.
 *
 * Renders a list of suggested tasks from an agent response.
 * Users can confirm (create tasks), edit suggestions, or dismiss them.
 */

import { useState, useCallback } from 'react';
import { CheckCircle, XCircle, Edit3, Plus, ChevronDown, ChevronUp, AlertCircle } from 'lucide-react';
import { cn } from '../../lib/utils';
import type { SuggestedTask, TaskSuggestionContent, TaskSuggestionStatus, Task } from '../../types';
import { confirmTaskSuggestions, dismissTaskSuggestions } from '../../lib/ipc';

interface TaskSuggestionCardProps {
  /** The message ID containing the suggestions */
  messageId: string;
  /** The channel ID */
  channelId: string;
  /** The raw JSON content string from the message */
  contentJson: string;
  /** Agent display name */
  agentName?: string;
  /** Callback when tasks are confirmed */
  onConfirmed?: (tasks: Task[]) => void;
  /** Callback when dismissed */
  onDismissed?: () => void;
}

/** Priority badge color map */
const priorityColors: Record<number, string> = {
  1: 'bg-red-100 text-red-700 border-red-300',
  2: 'bg-orange-100 text-orange-700 border-orange-300',
  3: 'bg-blue-100 text-blue-700 border-blue-300',
  4: 'bg-gray-100 text-gray-700 border-gray-300',
  5: 'bg-gray-50 text-gray-500 border-gray-200',
};

const priorityLabels: Record<number, string> = {
  1: 'Critical',
  2: 'High',
  3: 'Medium',
  4: 'Low',
  5: 'Trivial',
};

export function TaskSuggestionCard({
  messageId,
  channelId,
  contentJson,
  agentName: _agentName,
  onConfirmed,
  onDismissed,
}: TaskSuggestionCardProps) {
  // _agentName available for future use (e.g. attribution)
  void _agentName;
  const [isConfirming, setIsConfirming] = useState(false);
  const [isDismissing, setIsDismissing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expanded, setExpanded] = useState(true);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editValue, setEditValue] = useState<SuggestedTask | null>(null);
  const [localSuggestions, setLocalSuggestions] = useState<SuggestedTask[]>([]);

  // Parse content
  let parsed: TaskSuggestionContent | null = null;
  try {
    parsed = JSON.parse(contentJson);
  } catch {
    return (
      <div className="rounded-lg border border-red-300 bg-red-50 p-3 text-sm text-red-700">
        <AlertCircle size={16} className="inline mr-1" />
        Failed to parse task suggestions
      </div>
    );
  }

  if (!parsed || (parsed as unknown as Record<string, unknown>).type !== 'task_suggestion') {
    return null;
  }

  const suggestions = localSuggestions.length > 0 ? localSuggestions : parsed.suggestions;
  const status: TaskSuggestionStatus = parsed.status;
  const isPending = status === 'pending';
  const isConfirmed = status === 'confirmed';
  const isDismissed = status === 'dismissed';

  const handleConfirm = useCallback(async () => {
    setIsConfirming(true);
    setError(null);
    try {
      const tasks = await confirmTaskSuggestions(messageId, channelId, suggestions);
      onConfirmed?.(tasks);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsConfirming(false);
    }
  }, [messageId, channelId, suggestions, onConfirmed]);

  const handleDismiss = useCallback(async () => {
    setIsDismissing(true);
    setError(null);
    try {
      await dismissTaskSuggestions(messageId, channelId);
      onDismissed?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setIsDismissing(false);
    }
  }, [messageId, channelId, onDismissed]);

  const handleEditStart = (index: number) => {
    setEditingIndex(index);
    setEditValue({ ...suggestions[index] });
  };

  const handleEditSave = () => {
    if (editingIndex !== null && editValue) {
      const updated = [...suggestions];
      updated[editingIndex] = editValue;
      setLocalSuggestions(updated);
      setEditingIndex(null);
      setEditValue(null);
    }
  };

  const handleEditCancel = () => {
    setEditingIndex(null);
    setEditValue(null);
  };

  return (
    <div
      className={cn(
        'rounded-lg border-2 brutal-border overflow-hidden',
        isConfirmed && 'bg-green-50 border-green-300',
        isDismissed && 'bg-gray-50 border-gray-300 opacity-60',
        isPending && 'bg-amber-50 border-amber-300',
      )}
    >
      {/* Header */}
      <div
        className={cn(
          'flex items-center justify-between px-3 py-2 cursor-pointer',
          isConfirmed && 'bg-green-100',
          isDismissed && 'bg-gray-100',
          isPending && 'bg-amber-100',
        )}
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-center gap-2">
          <Plus size={14} className={isPending ? 'text-amber-600' : isConfirmed ? 'text-green-600' : 'text-gray-400'} />
          <span className="font-bold text-xs">
            {isPending && 'Task Suggestions'}
            {isConfirmed && 'Tasks Created'}
            {isDismissed && 'Suggestions Dismissed'}
          </span>
          <span className="text-[10px] text-gray-500">
            {suggestions.length} task{suggestions.length !== 1 ? 's' : ''}
          </span>
        </div>
        {expanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
      </div>

      {/* Suggestion list */}
      {expanded && (
        <div className="px-3 py-2 space-y-2">
          {suggestions.map((suggestion, idx) => (
            <div
              key={idx}
              className="rounded border bg-white p-2 text-xs space-y-1"
            >
              {editingIndex === idx ? (
                /* Edit mode */
                <div className="space-y-1">
                  <input
                    className="w-full border px-1.5 py-0.5 text-xs font-bold"
                    value={editValue?.title ?? ''}
                    onChange={(e) =>
                      setEditValue((prev) => prev ? { ...prev, title: e.target.value } : prev)
                    }
                  />
                  <textarea
                    className="w-full border px-1.5 py-0.5 text-xs"
                    rows={2}
                    value={editValue?.description ?? ''}
                    onChange={(e) =>
                      setEditValue((prev) => prev ? { ...prev, description: e.target.value } : prev)
                    }
                  />
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] text-gray-500">Priority:</span>
                    <select
                      className="border px-1 py-0.5 text-xs"
                      value={editValue?.priority ?? 3}
                      onChange={(e) =>
                        setEditValue((prev) => prev ? { ...prev, priority: Number(e.target.value) } : prev)
                      }
                    >
                      {[1, 2, 3, 4, 5].map((p) => (
                        <option key={p} value={p}>
                          {p} - {priorityLabels[p]}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="flex gap-1">
                    <button
                      className="px-2 py-0.5 bg-blue-500 text-white rounded text-[10px] font-bold"
                      onClick={handleEditSave}
                    >
                      Save
                    </button>
                    <button
                      className="px-2 py-0.5 bg-gray-200 rounded text-[10px]"
                      onClick={handleEditCancel}
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                /* Display mode */
                <>
                  <div className="flex items-start justify-between gap-2">
                    <span className="font-bold text-xs">{suggestion.title}</span>
                    <span
                      className={cn(
                        'px-1.5 py-0.5 rounded border text-[8px] font-bold shrink-0',
                        priorityColors[suggestion.priority] ?? priorityColors[3],
                      )}
                    >
                      P{suggestion.priority}
                    </span>
                  </div>
                  {suggestion.description && (
                    <p className="text-[10px] text-gray-600">{suggestion.description}</p>
                  )}
                  <div className="flex items-center gap-2 text-[10px] text-gray-500">
                    {suggestion.assignee && (
                      <span className="px-1 py-0.5 bg-purple-50 border border-purple-200 rounded">
                        @{suggestion.assignee}
                      </span>
                    )}
                  </div>
                </>
              )}
            </div>
          ))}

          {/* Action buttons (only in pending state) */}
          {isPending && (
            <div className="flex items-center gap-2 pt-1">
              <button
                className={cn(
                  'flex items-center gap-1 px-3 py-1.5 rounded text-xs font-bold',
                  'bg-green-500 text-white hover:bg-green-600 disabled:opacity-50',
                )}
                onClick={handleConfirm}
                disabled={isConfirming}
              >
                <CheckCircle size={12} />
                {isConfirming ? 'Creating...' : 'Confirm All'}
              </button>
              <button
                className="flex items-center gap-1 px-3 py-1.5 rounded text-xs font-bold bg-gray-200 hover:bg-gray-300"
                onClick={() => handleEditStart(0)}
              >
                <Edit3 size={12} />
                Edit
              </button>
              <button
                className={cn(
                  'flex items-center gap-1 px-3 py-1.5 rounded text-xs font-bold',
                  'bg-gray-100 text-gray-600 hover:bg-gray-200',
                )}
                onClick={handleDismiss}
                disabled={isDismissing}
              >
                <XCircle size={12} />
                {isDismissing ? '...' : 'Dismiss'}
              </button>
            </div>
          )}

          {/* Confirmed task IDs */}
          {isConfirmed && parsed.confirmed_task_ids && (
            <div className="text-[10px] text-green-700 pt-1">
              Created {parsed.confirmed_task_ids.length} task{parsed.confirmed_task_ids.length !== 1 ? 's' : ''}
            </div>
          )}

          {/* Error display */}
          {error && (
            <div className="text-[10px] text-red-600 bg-red-50 p-1 rounded">
              {error}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
