/**
 * Hook for managing Channel operations.
 *
 * Provides:
 * - Channel CRUD (create, list, get, update, delete)
 * - Channel member management (add, remove)
 * - Real-time multi-Agent streaming message send/receive in channels
 * - @Mention-aware agent resolution
 * - Per-agent streaming state tracking
 */

import { useState, useCallback, useRef, useEffect } from "react";
import type { Channel, ChannelInfo, AgentWithRuntime, ChannelChunkEvent, ChannelResponseEvent } from "../types";
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
  compactChannel,
} from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback detection
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

// ---------------------------------------------------------------------------
// Per-agent streaming state
// ---------------------------------------------------------------------------

/** Tracks the streaming state for a single agent in a multi-agent response. */
export interface AgentStreamState {
  agent_id: string;
  agent_index: number;
  total_agents: number;
  streaming: boolean;
  thinking: boolean;
  text: string;
  done: boolean;
  error?: string;
}

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface ChannelState {
  /** List of all channels */
  channels: ChannelInfo[];
  /** Currently active channel (null when no channel is selected) */
  activeChannel: Channel | null;
  /** Whether any agent is currently streaming */
  isStreaming: boolean;
  /** Whether any agent is "thinking" (waiting for first chunk) */
  isThinking: boolean;
  /** Buffered streaming text from the current single-agent response (backward compat) */
  streamingText: string;
  /** Per-agent streaming states for multi-agent responses */
  agentStreams: AgentStreamState[];
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
  /** Manually trigger compaction for a channel */
  compact: (channelId: string) => Promise<void>;
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
  const [agentStreams, setAgentStreams] = useState<AgentStreamState[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const unlistenChunkRef = useRef<(() => void) | null>(null);
  const unlistenResponseRef = useRef<(() => void) | null>(null);
  const unlistenAgentStartRef = useRef<(() => void) | null>(null);
  const unlistenRuntimeErrorRef = useRef<(() => void) | null>(null);

  // Clean up listeners on unmount
  useEffect(() => {
    return () => {
      if (unlistenChunkRef.current) unlistenChunkRef.current();
      if (unlistenResponseRef.current) unlistenResponseRef.current();
      if (unlistenAgentStartRef.current) unlistenAgentStartRef.current();
      if (unlistenRuntimeErrorRef.current) unlistenRuntimeErrorRef.current();
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
        const mockChannel: Channel = {
          id: `ch-dev-${Date.now()}`,
          name,
          members: memberAgentIds.map((aid) => ({
            agent_id: aid,
            role: "member",
            joined_at: new Date().toISOString(),
          })),
          messages: [],
          summary: null,
          summary_up_to: null,
          summary_updated_at: null,
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
          summary: null,
          summary_up_to: null,
          summary_updated_at: null,
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
    setAgentStreams([]);
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
      if (unlistenAgentStartRef.current) {
        unlistenAgentStartRef.current();
        unlistenAgentStartRef.current = null;
      }
      if (unlistenRuntimeErrorRef.current) {
        unlistenRuntimeErrorRef.current();
        unlistenRuntimeErrorRef.current = null;
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
        setIsThinking(false);
        return;
      }

      // Real Tauri flow
      const updatedChannel = await sendChannelMessage(channelId, message);
      setActiveChannel(updatedChannel);

      // Listen for agent-start events (multi-agent coordination)
      const { listen } = await import("@tauri-apps/api/event");

      const unlistenAgentStart = await listen(
        "agent://channel-agent-start",
        (event: { payload: { channel_id: string; agent_id: string; agent_index: number; total_agents: number } }) => {
          const payload = event.payload;
          if (payload.channel_id !== channelId) return;

          setAgentStreams((prev) => [
            ...prev,
            {
              agent_id: payload.agent_id,
              agent_index: payload.agent_index,
              total_agents: payload.total_agents,
              streaming: true,
              thinking: true,
              text: "",
              done: false,
            },
          ]);
        }
      );
      unlistenAgentStartRef.current = unlistenAgentStart;

      // Listen for runtime unavailability events (rich error with install hint)
      const unlistenRuntimeError = await listen<{
        channel_id: string;
        agent_id: string;
        runtime_id: string;
        runtime_name: string;
        install_hint: string;
        error: string;
      }>("runtime://unavailable", async (event) => {
        const { channel_id: evtChannelId, runtime_name, install_hint, error: runtimeError } = event.payload;
        if (evtChannelId !== channelId) return;

        setError(`${runtime_name}: ${runtimeError}${install_hint ? `\nInstall: ${install_hint}` : ""}`);
        setIsStreaming(false);
        setIsThinking(false);

        unlistenRuntimeError();
        unlistenRuntimeErrorRef.current = null;
      });
      unlistenRuntimeErrorRef.current = unlistenRuntimeError;

      // Per-agent accumulated text
      const agentTexts = new Map<string, string>();

      // Listen for streaming chunk events
      const unlistenChunk = await listen(
        "agent://channel-chunk",
        (event: { payload: ChannelChunkEvent }) => {
          const payload = event.payload;
          if (payload.channel_id !== channelId) return;

          const streamEvent = payload.event;
          const agentId = payload.agent_id;

          if ((streamEvent as unknown as Record<string, unknown>).type === "assistant" && streamEvent.text) {
            // Update per-agent text
            const prev = agentTexts.get(agentId) || "";
            const newText = prev + streamEvent.text;
            agentTexts.set(agentId, newText);

            // Update per-agent stream state
            setAgentStreams((prev) =>
              prev.map((s) =>
                s.agent_id === agentId
                  ? { ...s, thinking: false, text: newText }
                  : s
              )
            );

            // Also update single-agent compat streaming text
            setStreamingText(newText);
            setIsThinking(false);
          }

          if (streamEvent.is_done) {
            setAgentStreams((prev) =>
              prev.map((s) =>
                s.agent_id === agentId
                  ? { ...s, streaming: false, thinking: false, done: true, error: streamEvent.error }
                  : s
              )
            );
          }
        }
      );
      unlistenChunkRef.current = unlistenChunk;

      // Listen for the channel-response event
      const unlistenResponse = await listen(
        "agent://channel-response",
        async (event: { payload: ChannelResponseEvent }) => {
          const { channel_id, agent_id, content } = event.payload;
          if (channel_id !== channelId) return;

          try {
            const finalChannel = await saveChannelResponse(channel_id, agent_id, content);
            setActiveChannel(finalChannel);
          } catch (err) {
            console.error("[useChannel] save_channel_response failed:", err);
          }

          // Check if all agents are done
          setAgentStreams((prev) => {
            const allDone = prev.every((s) => s.done);
            if (allDone) {
              setStreamingText("");
              setIsStreaming(false);
              setIsThinking(false);

              // Clean up listeners
              unlistenAgentStart();
              unlistenChunk();
              unlistenResponse();
              unlistenRuntimeError();
              unlistenChunkRef.current = null;
              unlistenResponseRef.current = null;
              unlistenAgentStartRef.current = null;
              unlistenRuntimeErrorRef.current = null;

              // Refresh channel list
              loadChannels();
            }
            return allDone ? [] : prev;
          });
        }
      );
      unlistenResponseRef.current = unlistenResponse;

      // Fallback timeout: if no agent-start event within 5s, consider it single-agent
      setTimeout(() => {
        setAgentStreams((prev) => {
          if (prev.length === 0 && isStreaming) {
            setIsStreaming(false);
            setIsThinking(false);
            setStreamingText("");
          }
          return prev;
        });
      }, 30000);
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
    setAgentStreams([]);
  }, []);

  /** Manually trigger compaction for a channel */
  const compact = useCallback(async (channelId: string) => {
    if (!isTauri) return;
    try {
      const updated = await compactChannel(channelId);
      if (activeChannel?.id === channelId) {
        setActiveChannel(updated);
      }
    } catch (err) {
      console.error("[useChannel] compact failed:", err);
    }
  }, [activeChannel]);

  return {
    channels,
    activeChannel,
    isStreaming,
    isThinking,
    streamingText,
    agentStreams,
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
    compact,
    clearActive,
  };
}
