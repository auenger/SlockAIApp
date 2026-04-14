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
import type { Channel, ChannelInfo, ChannelMessage, AgentWithRuntime, ChannelChunkEvent, ChannelResponseEvent, ChannelA2aStartEvent, ChannelA2aDepthExceededEvent, ContentBlock } from "../types";
import {
  createChannel,
  listChannels,
  getChannel,
  updateChannel,
  deleteChannel,
  addChannelMember,
  removeChannelMember,
  sendChannelMessage,
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
  /** Whether this agent was triggered by another agent (A2A) rather than directly by the user */
  is_a2a?: boolean;
  /** The agent that triggered this agent via @{agent} mention (only set for A2A) */
  triggered_by?: string;
  /** Depth in the A2A trigger chain (0 = user-triggered, 1+ = A2A) */
  a2a_depth?: number;
  /** Structured content blocks (tool_use / tool_result) collected during streaming. Cleared on done. */
  contentBlocks: ContentBlock[];
  /** Status message from system events (e.g., "Session initialized · claude-sonnet-4") */
  statusMessage?: string;
}

// ---------------------------------------------------------------------------
// Per-channel streaming state
// ---------------------------------------------------------------------------

/** Streaming state for a single channel — stored in a Map keyed by channelId. */
export interface ChannelStreamState {
  isStreaming: boolean;
  isThinking: boolean;
  streamingText: string;
  agentStreams: AgentStreamState[];
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
  send: (channelId: string, message: string, agents: AgentWithRuntime[], userName?: string) => Promise<void>;
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
  const [channelStreamStates, setChannelStreamStates] = useState<Map<string, ChannelStreamState>>(new Map());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // ---------------------------------------------------------------------------
  // Per-channel state helpers
  // ---------------------------------------------------------------------------

  /** Get the streaming state for a specific channel (defaults to idle). */
  const getStreamState = useCallback((channelId: string): ChannelStreamState => {
    return channelStreamStates.get(channelId) ?? {
      isStreaming: false,
      isThinking: false,
      streamingText: "",
      agentStreams: [],
    };
  }, [channelStreamStates]);

  /** Merge partial state into a specific channel's Map entry. */
  const setStreamState = useCallback((channelId: string, partial: Partial<ChannelStreamState>) => {
    setChannelStreamStates((prev) => {
      const next = new Map(prev);
      const current = next.get(channelId) ?? {
        isStreaming: false,
        isThinking: false,
        streamingText: "",
        agentStreams: [],
      };
      next.set(channelId, { ...current, ...partial });
      return next;
    });
  }, []);

  /** Update agentStreams for a specific channel via transformer function. */
  const setChannelAgentStreams = useCallback((channelId: string, updater: (prev: AgentStreamState[]) => AgentStreamState[]) => {
    setChannelStreamStates((prev) => {
      const next = new Map(prev);
      const current = next.get(channelId) ?? {
        isStreaming: false,
        isThinking: false,
        streamingText: "",
        agentStreams: [],
      };
      next.set(channelId, { ...current, agentStreams: updater(current.agentStreams) });
      return next;
    });
  }, []);

  /** Clear the streaming state for a specific channel (on session complete). */
  const clearStreamState = useCallback((channelId: string) => {
    setChannelStreamStates((prev) => {
      const next = new Map(prev);
      next.delete(channelId);
      return next;
    });
  }, []);

  // Derive the active channel's streaming state for the hook's return value
  const activeChannelId = activeChannel?.id ?? null;
  const activeStreamState = activeChannelId ? getStreamState(activeChannelId) : null;
  const isStreaming = activeStreamState?.isStreaming ?? false;
  const isThinking = activeStreamState?.isThinking ?? false;
  const streamingText = activeStreamState?.streamingText ?? "";
  const agentStreams = activeStreamState?.agentStreams ?? [];

  const unlistenChunkRef = useRef<(() => void) | null>(null);
  const unlistenResponseRef = useRef<(() => void) | null>(null);
  const unlistenAgentStartRef = useRef<(() => void) | null>(null);
  const unlistenRuntimeErrorRef = useRef<(() => void) | null>(null);
  const unlistenA2aStartRef = useRef<(() => void) | null>(null);
  const unlistenA2aDepthRef = useRef<(() => void) | null>(null);
  const unlistenSessionCompleteRef = useRef<(() => void) | null>(null);

  // Ref guard to prevent concurrent send calls
  const isSendingRef = useRef(false);

  // Clean up listeners on unmount
  useEffect(() => {
    return () => {
      if (unlistenChunkRef.current) unlistenChunkRef.current();
      if (unlistenResponseRef.current) unlistenResponseRef.current();
      if (unlistenAgentStartRef.current) unlistenAgentStartRef.current();
      if (unlistenRuntimeErrorRef.current) unlistenRuntimeErrorRef.current();
      if (unlistenA2aStartRef.current) unlistenA2aStartRef.current();
      if (unlistenA2aDepthRef.current) unlistenA2aDepthRef.current();
      if (unlistenSessionCompleteRef.current) unlistenSessionCompleteRef.current();
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
  const send = useCallback(async (channelId: string, message: string, _agents: AgentWithRuntime[], userName?: string) => {
    // Prevent concurrent sends — ref guard is synchronous, unlike React state
    if (isSendingRef.current) return;
    isSendingRef.current = true;
    setError(null);

    try {
      // Clean up previous listeners
      if (unlistenChunkRef.current) { unlistenChunkRef.current(); unlistenChunkRef.current = null; }
      if (unlistenResponseRef.current) { unlistenResponseRef.current(); unlistenResponseRef.current = null; }
      if (unlistenAgentStartRef.current) { unlistenAgentStartRef.current(); unlistenAgentStartRef.current = null; }
      if (unlistenRuntimeErrorRef.current) { unlistenRuntimeErrorRef.current(); unlistenRuntimeErrorRef.current = null; }
      if (unlistenA2aStartRef.current) { unlistenA2aStartRef.current(); unlistenA2aStartRef.current = null; }
      if (unlistenA2aDepthRef.current) { unlistenA2aDepthRef.current(); unlistenA2aDepthRef.current = null; }

      if (!isTauri) {
        // Dev fallback: simulate streaming
        setStreamState(channelId, { isThinking: false, isStreaming: true, streamingText: "", agentStreams: [] });

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
          setStreamState(channelId, { streamingText: mockResponse.substring(0, i + 1) });
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
        clearStreamState(channelId);
        return;
      }

      // =====================================================================
      // Real Tauri flow
      // =====================================================================
      //
      // CRITICAL: Event listeners MUST be set up BEFORE the IPC call.
      // The backend emits events during agent execution, and if listeners
      // aren't registered yet, those events are lost (no real-time rendering).
      //
      // Flow:
      // 1. Set up all event listeners
      // 2. Set streaming UI state + optimistic user message
      // 3. Call IPC (backend executes agents, emits events → listeners handle)
      // 4. On IPC return, replace state with final backend data
      // 5. If no agents triggered → clean up streaming state immediately

      const { listen } = await import("@tauri-apps/api/event");

      // Per-agent accumulated text
      const agentTexts = new Map<string, string>();

      // Track expected agent count for UI display purposes.
      // Actual session completion is driven by backend's session-complete event.
      let expectedAgentCount = 0;

      // Cleanup function: tears down all streaming listeners and resets UI state.
      // Called only when the backend confirms the entire session (including A2A chain) is done.
      let sessionCompleteFired = false;
      const cleanupSession = (
        unlistenAgentStart: () => void,
        unlistenChunk: () => void,
        unlistenResponse: () => void,
        unlistenRuntimeError: () => void,
        unlistenA2aStart: () => void,
        unlistenA2aDepth: () => void,
        unlistenSessionComplete?: () => void,
      ) => {
        if (sessionCompleteFired) return; // Guard against double-cleanup
        sessionCompleteFired = true;

        // Clear only this channel's streaming state
        clearStreamState(channelId);

        unlistenAgentStart();
        unlistenChunk();
        unlistenResponse();
        unlistenRuntimeError();
        unlistenA2aStart();
        unlistenA2aDepth();
        unlistenSessionComplete?.();
        unlistenChunkRef.current = null;
        unlistenResponseRef.current = null;
        unlistenAgentStartRef.current = null;
        unlistenRuntimeErrorRef.current = null;
        unlistenA2aStartRef.current = null;
        unlistenA2aDepthRef.current = null;
        unlistenSessionCompleteRef.current = null;

        loadChannels();
      };

      // --- Step 1: Set up listeners BEFORE IPC call ---

      const unlistenAgentStart = await listen(
        "agent://channel-agent-start",
        (event: { payload: { channel_id: string; agent_id: string; agent_index: number; total_agents: number } }) => {
          const payload = event.payload;
          if (payload.channel_id !== channelId) return;

          setChannelAgentStreams(channelId, (prev) => [
            ...prev,
            {
              agent_id: payload.agent_id,
              agent_index: payload.agent_index,
              total_agents: payload.total_agents,
              streaming: true,
              thinking: true,
              text: "",
              done: false,
              contentBlocks: [],
            },
          ]);
        }
      );
      unlistenAgentStartRef.current = unlistenAgentStart;

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
        setStreamState(channelId, { isStreaming: false, isThinking: false });

        unlistenRuntimeError();
        unlistenRuntimeErrorRef.current = null;
      });
      unlistenRuntimeErrorRef.current = unlistenRuntimeError;

      const unlistenA2aStart = await listen(
        "agent://channel-a2a-start",
        (event: { payload: ChannelA2aStartEvent }) => {
          const payload = event.payload;
          if (payload.channel_id !== channelId) return;

          // Increment expected count — this A2A agent was not counted in the
          // initial agents_triggered from IPC response.
          expectedAgentCount += 1;

          // Update the existing entry created by agent://channel-agent-start
          // instead of creating a duplicate.
          setChannelAgentStreams(channelId, (prev) => {
            const existing = prev.findIndex((s) => s.agent_id === payload.agent_id);
            if (existing >= 0) {
              // Entry already exists from agent-start — update with A2A metadata
              return prev.map((s, i) =>
                i === existing
                  ? {
                      ...s,
                      is_a2a: true,
                      triggered_by: payload.triggered_by,
                      a2a_depth: payload.depth,
                    }
                  : s
              );
            }
            // Fallback: entry doesn't exist yet (unlikely, but safe)
            return [
              ...prev,
              {
                agent_id: payload.agent_id,
                agent_index: prev.length,
                total_agents: prev.length + 1,
                streaming: true,
                thinking: true,
                text: "",
                done: false,
                is_a2a: true,
                triggered_by: payload.triggered_by,
                a2a_depth: payload.depth,
                contentBlocks: [],
              },
            ];
          });
        }
      );
      unlistenA2aStartRef.current = unlistenA2aStart;

      const unlistenA2aDepth = await listen(
        "agent://channel-a2a-depth-exceeded",
        (event: { payload: ChannelA2aDepthExceededEvent }) => {
          const payload = event.payload;
          if (payload.channel_id !== channelId) return;

          console.warn(
            `[useChannel] A2A trigger chain depth exceeded: ${payload.triggered_by} -> @${payload.agent_id} at depth ${payload.depth}/${payload.max_depth}`
          );
        }
      );
      unlistenA2aDepthRef.current = unlistenA2aDepth;

      const unlistenChunk = await listen(
        "agent://channel-chunk",
        (event: { payload: ChannelChunkEvent }) => {
          const payload = event.payload;
          if (payload.channel_id !== channelId) return;

          const streamEvent = payload.event;
          const agentId = payload.agent_id;

          const eventType = (streamEvent as unknown as Record<string, unknown>).type as string;
          const newBlocks: ContentBlock[] = streamEvent.content_blocks ?? [];

          if (eventType === "assistant") {
            if (streamEvent.text) {
              // Assistant event with text content — append to accumulated text
              const prev = agentTexts.get(agentId) || "";
              const newText = prev + streamEvent.text;
              agentTexts.set(agentId, newText);

              // Update per-agent stream state
              setChannelAgentStreams(channelId, (prev) =>
                prev.map((s) =>
                  s.agent_id === agentId
                    ? { ...s, thinking: false, text: newText, contentBlocks: [...s.contentBlocks, ...newBlocks], statusMessage: undefined }
                    : s
                )
              );

              setStreamState(channelId, { streamingText: newText, isThinking: false });
            } else if (newBlocks.length > 0) {
              // Assistant event with content_blocks but no text (e.g. tool_use only).
              // Update content blocks so the user can see tool calls during thinking.
              setChannelAgentStreams(channelId, (prev) =>
                prev.map((s) =>
                  s.agent_id === agentId
                    ? { ...s, contentBlocks: [...s.contentBlocks, ...newBlocks] }
                    : s
                )
              );
            }
          } else if (eventType === "user") {
            // User events carry tool_result blocks from CLI tool execution.
            if (newBlocks.length > 0) {
              setChannelAgentStreams(channelId, (prev) =>
                prev.map((s) =>
                  s.agent_id === agentId
                    ? { ...s, contentBlocks: [...s.contentBlocks, ...newBlocks] }
                    : s
                )
              );
            }
          } else if (eventType === "system") {
            // System events carry initialization status.
            if (streamEvent.text) {
              setChannelAgentStreams(channelId, (prev) =>
                prev.map((s) =>
                  s.agent_id === agentId
                    ? { ...s, statusMessage: streamEvent.text }
                    : s
                )
              );
            }
          }

          if (streamEvent.is_done) {
            // Clear contentBlocks on done (not persisted)
            setChannelAgentStreams(channelId, (prev) =>
              prev.map((s) =>
                s.agent_id === agentId
                  ? { ...s, streaming: false, thinking: false, done: true, error: streamEvent.error, contentBlocks: [], statusMessage: undefined }
                  : s
              )
            );
          }
        }
      );
      unlistenChunkRef.current = unlistenChunk;

      // channel-response: update UI with completed agent message.
      // Do NOT call saveChannelResponse — the backend already saves in execute_single_agent.
      const unlistenResponse = await listen(
        "agent://channel-response",
        async (event: { payload: ChannelResponseEvent }) => {
          const { channel_id, agent_id, content, content_blocks } = event.payload;
          if (channel_id !== channelId) return;

          // Add agent message to UI state (backend already persisted it)
          setActiveChannel((prev) => {
            if (!prev || prev.id !== channel_id) return prev;
            const exists = prev.messages.some(
              (m) => m.sender_type === "agent" && m.sender_id === agent_id && m.content === content
            );
            if (exists) return prev;
            return {
              ...prev,
              messages: [
                ...prev.messages,
                {
                  id: `msg-${Date.now()}`,
                  channel_id,
                  sender_type: "agent" as const,
                  sender_id: agent_id,
                  content,
                  content_blocks,
                  timestamp: new Date().toISOString(),
                },
              ],
            };
          });

          // Mark this agent as done (session cleanup is deferred to session-complete event)
          setChannelAgentStreams(channelId, (prev) =>
            prev.map((s) =>
              s.agent_id === agent_id
                ? { ...s, streaming: false, thinking: false, done: true }
                : s
            )
          );
        }
      );
      unlistenResponseRef.current = unlistenResponse;

      // session-complete: backend confirms ALL agents (including A2A chain) are done.
      // This is the ONLY signal that triggers listener cleanup.
      const unlistenSessionComplete = await listen(
        "agent://channel-session-complete",
        (event: { payload: { channel_id: string } }) => {
          if (event.payload.channel_id !== channelId) return;
          cleanupSession(
            unlistenAgentStart, unlistenChunk, unlistenResponse,
            unlistenRuntimeError, unlistenA2aStart, unlistenA2aDepth,
            unlistenSessionComplete,
          );
        }
      );
      unlistenSessionCompleteRef.current = unlistenSessionComplete;

      // --- Step 2: Set streaming UI state + optimistic user message ---

      setStreamState(channelId, { isThinking: true, isStreaming: true, streamingText: "", agentStreams: [] });

      const optimisticUserMsg: ChannelMessage = {
        id: `msg-pending-${Date.now()}`,
        channel_id: channelId,
        sender_type: "user",
        sender_id: "user",
        content: message,
        timestamp: new Date().toISOString(),
      };
      setActiveChannel((prev) => {
        if (!prev || prev.id !== channelId) return prev;
        return { ...prev, messages: [...prev.messages, optimisticUserMsg] };
      });

      // --- Step 3: IPC call — returns immediately with user message saved ---

      const response = await sendChannelMessage(channelId, message, userName);

      // --- Step 4: Replace optimistic data with real backend data ---
      // Backend now returns immediately (agent execution runs in background).
      // The channel only has the user message; agent responses arrive via events.

      setActiveChannel(response.channel);

      // --- Step 5: Handle no-agent-triggered case ---
      // The backend tells us how many agents were triggered.
      // If 0 (no @mentions), clean up streaming state immediately.

      if (response.agents_triggered === 0) {
        // No @mentions → no agents were triggered. Clean up streaming state.
        cleanupSession(
          unlistenAgentStart, unlistenChunk, unlistenResponse,
          unlistenRuntimeError, unlistenA2aStart, unlistenA2aDepth,
          unlistenSessionComplete,
        );
      } else {
        // Initialize expected agent count from IPC response.
        // A2A agents will increment this later via the a2a-start listener.
        expectedAgentCount = response.agents_triggered;
      }

      // No timeout — state is driven entirely by events:
      // - channel-response → marks agent done, triggers cleanup when all complete
      // - runtime://unavailable → handles runtime crashes
      // - Stop button → user can manually interrupt a stuck session
      // As long as the runtime is alive, the UI reflects it.
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      setStreamState(channelId, { isStreaming: false, isThinking: false });
    } finally {
      isSendingRef.current = false;
    }
  }, [loadChannels, setStreamState, clearStreamState, setChannelAgentStreams]);

  /** Clear the active channel */
  const clearActive = useCallback(() => {
    if (activeChannelId) {
      clearStreamState(activeChannelId);
    }
    setActiveChannel(null);
  }, [activeChannelId, clearStreamState]);

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
