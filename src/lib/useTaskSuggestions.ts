/**
 * Hook for managing task suggestions in channel conversations.
 *
 * Listens for task://suggested events and provides methods
 * to confirm or dismiss suggestions.
 */

import { useState, useCallback, useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { TaskSuggestedEvent } from '../types';
import { confirmTaskSuggestions, dismissTaskSuggestions } from './ipc';
import type { SuggestedTask, Task } from '../types';

/**
 * Hook that manages task suggestion interactions within a channel.
 *
 * @param channelId - The active channel ID to filter events for
 */
export function useTaskSuggestions(channelId: string | null) {
  const [pendingSuggestions, setPendingSuggestions] = useState<
    Map<string, { messageId: string; agentId: string; channelId: string }>
  >(new Map());

  // Listen for task://suggested events
  useEffect(() => {
    if (!channelId) return;

    let unlisten: UnlistenFn | null = null;

    const setup = async () => {
      unlisten = await listen<TaskSuggestedEvent>('task://suggested', (event) => {
        if (event.payload.channel_id === channelId) {
          setPendingSuggestions((prev) => {
            const next = new Map(prev);
            next.set(event.payload.message_id, {
              messageId: event.payload.message_id,
              agentId: event.payload.agent_id,
              channelId: event.payload.channel_id,
            });
            return next;
          });
        }
      });
    };

    setup();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [channelId]);

  // Listen for task://suggested-confirmed to remove from pending
  useEffect(() => {
    if (!channelId) return;

    let unlisten: UnlistenFn | null = null;

    const setup = async () => {
      unlisten = await listen<{ channel_id: string; message_id: string }>(
        'task://suggested-confirmed',
        (event) => {
          if (event.payload.channel_id === channelId) {
            setPendingSuggestions((prev) => {
              const next = new Map(prev);
              next.delete(event.payload.message_id);
              return next;
            });
          }
        }
      );
    };

    setup();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [channelId]);

  // Listen for task://suggested-dismissed to remove from pending
  useEffect(() => {
    if (!channelId) return;

    let unlisten: UnlistenFn | null = null;

    const setup = async () => {
      unlisten = await listen<{ channel_id: string; message_id: string }>(
        'task://suggested-dismissed',
        (event) => {
          if (event.payload.channel_id === channelId) {
            setPendingSuggestions((prev) => {
              const next = new Map(prev);
              next.delete(event.payload.message_id);
              return next;
            });
          }
        }
      );
    };

    setup();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [channelId]);

  /** Confirm selected task suggestions */
  const confirm = useCallback(
    async (messageId: string, selected: SuggestedTask[]): Promise<Task[]> => {
      if (!channelId) return [];
      const tasks = await confirmTaskSuggestions(messageId, channelId, selected);
      return tasks;
    },
    [channelId]
  );

  /** Dismiss task suggestions */
  const dismiss = useCallback(
    async (messageId: string): Promise<void> => {
      if (!channelId) return;
      await dismissTaskSuggestions(messageId, channelId);
    },
    [channelId]
  );

  return {
    pendingSuggestions,
    confirm,
    dismiss,
  };
}
