/**
 * Hook for managing Channel operations.
 *
 * Provides:
 * - Channel CRUD (create, list, get, update, delete)
 * - Channel member management (add, remove)
 * - Real-time streaming message send/receive in channels
 * - State management for active channel and messages
 */

import { useState, useCallback, useRef, useEffect } from "react";
import type { Channel, ChannelInfo, StreamEvent, AgentWithRuntime } from "../types";
import {
  createChannel,
  listChannels,
  getChannel,
  updateChannel,
  deleteChannel,
  addChannelMember,
  removeChannelMember,
  sendChannelMessage,
  saveChannelResponse,
} from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback detection
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface ChannelState {
  /** List of all channels */
  channels: ChannelInfo[];
  /** Currently active channel (null when no channel is selected) */
  activeChannel: Channel | null;
  /** Whether a message is currently being streamed */
  isStreaming: boolean;
  /** Whether the agent is "thinking" (streaming in progress) */
  isThinking: boolean;
  /** Buffered streaming text from the current response */
  streamingText: string;
  /** Loading state for async operations */
  loading: boolean;
  /** Error message (if any) */
  error: string | null;

  // Actions
  /** Load all channels */
  loadChannels: () => Promise<void>;
  /** Create a new channel */
  create: (name: string, memberAgentIds: string[]) => Promise<Channel>;
  /** Select a channel (loads its full data) */
  selectChannel: (channelId: string) => Promise<void>;
  /** Update a channel's name */
  update: (channelId: string, name: string) => Promise<void>;
  /** Delete a channel */
  remove: (channelId: string) => Promise<void>;
  /** Add an agent member to a channel */
  addMember: (channelId: string, agentId: string) => Promise<void>;
  /** Remove an agent member from a channel */
  removeMember: (channelId: string, agentId: string) => Promise<void>;
  /** Send a message in the active channel */
  send: (channelId: string, message: string, agents: AgentWithRuntime[]) => Promise<void>;
  /** Clear the active channel */
  clearActive: () => void;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useChannel(): ChannelState {
  const [channels, setChannels] = useState<ChannelInfo[]>([]);
  const [activeChannel, setActiveChannel] = useState<Channel | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const [isThinking, setIsThinking] = useState(false);
  const [streamingText, setStreamingText] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const unlistenChunkRef = useRef<(() => void) | null>(null);
  const unlistenResponseRef = useRef<(() => void) | null>(null);

  // Clean up listeners on unmount
  useEffect(() => {
    return () => {
      if (unlistenChunkRef.current) unlistenChunkRef.current();
      if (unlistenResponseRef.current) unlistenResponseRef.current();
    };
  }, []);

  /** Load all channels */
  const loadChannels = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        // Dev fallback: mock data
        setChannels([
          {
            id: "dev-channel-1",
            name: "all",
            member_count: 2,
            unread_count: 0,
            preview: "",
            message_count: 0,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ]);
        return;
      }
      const list = await listChannels();
      setChannels(list);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  /** Create a new channel */
  const create = useCallback(async (name: string, memberAgentIds: string[]): Promise<Channel> => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        // Dev fallback
        const mockChannel: Channel = {
          id: `ch-dev-${Date.now()}`,
          name,
          members: memberAgentIds.map((aid) => ({
            agent_id: aid,
            role: "member",
            joined_at: new Date().toISOString(),
          })),
          messages: [],
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        setActiveChannel(mockChannel);
        await loadChannels();
        return mockChannel;
      }
      const channel = await createChannel(name, memberAgentIds);
      await loadChannels();
      return channel;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [loadChannels]);

  /** Select a channel */
  const selectChannel = useCallback(async (channelId: string) => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        setActiveChannel({
          id: channelId,
          name: channelId === "dev-channel-1" ? "all" : "channel",
          members: [],
          messages: [],
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        });
        return;
      }
      const channel = await getChannel(channelId);
      setActiveChannel(channel);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  /** Update a channel */
  const update = useCallback(async (channelId: string, name: string) => {
    try {
      if (isTauri) {
        await updateChannel(channelId, name);
      }
      await loadChannels();
      // Refresh active channel if it's the one updated
      setActiveChannel((prev) => {
        if (prev && prev.id === channelId) {
          return { ...prev, name };
        }
        return prev;
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    }
  }, [loadChannels]);

  /** Delete a channel */
  const remove = useCallback(async (channelId: string) => {
    try {
      if (isTauri) {
        await deleteChannel(channelId);
      }
      if (activeChannel?.id === channelId) {
        setActiveChannel(null);
      }
      await loadChannels();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    }
  }, [activeChannel, loadChannels]);

  /** Add a member to a channel */
  const addMember = useCallback(async (channelId: string, agentId: string) => {
    try {
      if (isTauri) {
        const updated = await addChannelMember(channelId, agentId);
        if (activeChannel?.id === channelId) {
          setActiveChannel(updated);
        }
      }
      await loadChannels();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    }
  }, [activeChannel, loadChannels]);

  /** Remove a member from a channel */
  const removeMember = useCallback(async (channelId: string, agentId: string) => {
    try {
      if (isTauri) {
        const updated = await removeChannelMember(channelId, agentId);
        if (activeChannel?.id === channelId) {
          setActiveChannel(updated);
        }
      }
      await loadChannels();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    }
  }, [activeChannel, loadChannels]);

  /** Send a message in a channel */
  const send = useCallback(async (channelId: string, message: string, _agents: AgentWithRuntime[]) => {
    setIsThinking(true);
    setIsStreaming(true);
    setStreamingText("");
    setError(null);

    try {
      // Clean up previous listeners
      if (unlistenChunkRef.current) {
        unlistenChunkRef.current();
        unlistenChunkRef.current = null;
      }
      if (unlistenResponseRef.current) {
        unlistenResponseRef.current();
        unlistenResponseRef.current = null;
      }

      if (!isTauri) {
        // Dev fallback: simulate streaming
        setIsThinking(false);

        setActiveChannel((prev) => {
          if (!prev || prev.id !== channelId) return prev;
          return {
            ...prev,
            messages: [
              ...prev.messages,
              {
                id: `msg-dev-${Date.now()}`,
                channel_id: channelId,
                sender_type: "user" as const,
                sender_id: "user",
                content: message,
                timestamp: new Date().toISOString(),
              },
            ],
          };
        });

        const mockResponse = `[Dev Mode] Channel received: "${message}". This is a simulated channel response.`;
        for (let i = 0; i < mockResponse.length; i++) {
          await new Promise((r) => setTimeout(r, 20));
          setStreamingText(mockResponse.substring(0, i + 1));
        }

        setActiveChannel((prev) => {
          if (!prev || prev.id !== channelId) return prev;
          return {
            ...prev,
            messages: [
              ...prev.messages,
              {
                id: `msg-dev-${Date.now() + 1}`,
                channel_id: channelId,
                sender_type: "agent" as const,
                sender_id: prev.members[0]?.agent_id || "agent",
                content: mockResponse,
                timestamp: new Date().toISOString(),
              },
            ],
          };
        });
        setStreamingText("");
        setIsStreaming(false);
        return;
      }

      // Real Tauri flow
      const updatedChannel = await sendChannelMessage(channelId, message);
      setActiveChannel(updatedChannel);

      // Listen for streaming chunk events
      const { listen } = await import("@tauri-apps/api/event");

      let accumulatedText = "";

      const unlistenChunk = await listen<{ channel_id: string; event: StreamEvent }>(
        "agent://channel-chunk",
        (event) => {
          const payload = event.payload;
          const streamEvent = payload.event;
          if ((streamEvent as unknown as Record<string, unknown>).msg_type === "assistant" && streamEvent.text) {
            accumulatedText += streamEvent.text;
            setStreamingText(accumulatedText);
            setIsThinking(false);
          }
          if (streamEvent.is_done) {
            setIsStreaming(false);
            setIsThinking(false);
          }
        }
      );
      unlistenChunkRef.current = unlistenChunk;

      // Listen for the channel-response event
      const unlistenResponse = await listen<{
        channel_id: string;
        agent_id: string;
        content: string;
        session_id: string | null;
      }>("agent://channel-response", async (event) => {
        const { channel_id, agent_id, content } = event.payload;

        try {
          const finalChannel = await saveChannelResponse(channel_id, agent_id, content);
          setActiveChannel(finalChannel);
        } catch (err) {
          console.error("[useChannel] save_channel_response failed:", err);
        }

        setStreamingText("");

        unlistenChunk();
        unlistenResponse();
        unlistenChunkRef.current = null;
        unlistenResponseRef.current = null;

        // Refresh channel list (to update preview/message_count)
        await loadChannels();
      });
      unlistenResponseRef.current = unlistenResponse;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      setIsStreaming(false);
      setIsThinking(false);
    }
  }, [loadChannels]);

  /** Clear the active channel */
  const clearActive = useCallback(() => {
    setActiveChannel(null);
    setStreamingText("");
    setIsStreaming(false);
    setIsThinking(false);
  }, []);

  return {
    channels,
    activeChannel,
    isStreaming,
    isThinking,
    streamingText,
    loading,
    error,
    loadChannels,
    create,
    selectChannel,
    update,
    remove,
    addMember,
    removeMember,
    send,
    clearActive,
  };
}
