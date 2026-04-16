/**
 * Hook for managing Thread-based 1-on-1 Agent conversations.
 *
 * Provides:
 * - Thread CRUD (create, list, get, delete)
 * - Real-time streaming message send/receive
 * - Session resume support
 * - State management for active thread and messages
 */

import { useState, useCallback, useRef, useEffect } from "react";
import type { Thread, ThreadInfo, StreamEvent, ContentBlock } from "../types";
import {
  createThread,
  listThreads,
  getThread,
  deleteThread,
  sendMessage,
  saveAgentResponse,
  listAllThreads,
  renameThread,
} from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback detection
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface ThreadChatState {
  /** Currently active thread (null when no thread is selected) */
  activeThread: Thread | null;
  /** List of threads for the current agent */
  threads: ThreadInfo[];
  /** Whether a message is currently being streamed */
  isStreaming: boolean;
  /** Whether the agent is "thinking" (streaming in progress) */
  isThinking: boolean;
  /** Buffered streaming text from the current response */
  streamingText: string;
  /** Structured content blocks (tool_use / tool_result) collected during streaming */
  contentBlocks: ContentBlock[];
  /** Status message from system events (e.g., "Session initialized · claude-sonnet-4") */
  statusMessage: string;
  /** Whether the agent response is done */
  isDone: boolean;
  /** Loading state for async operations */
  loading: boolean;
  /** Error message (if any) */
  error: string | null;

  // Actions
  /** Create a new thread for the given agent */
  createNewThread: (agentId: string, agentName: string) => Promise<Thread>;
  /** Load threads list for an agent */
  loadThreads: (agentId: string) => Promise<void>;
  /** Load all threads across all agents */
  loadAllThreads: () => Promise<void>;
  /** Select a thread (loads its data) */
  selectThread: (agentId: string, threadId: string) => Promise<void>;
  /** Delete a thread */
  removeThread: (agentId: string, threadId: string) => Promise<void>;
  /** Rename a thread */
  renameThreadAction: (threadId: string, newTitle: string) => Promise<ThreadInfo | null>;
  /** Send a message in the active thread */
  send: (agentId: string, threadId: string, message: string) => Promise<void>;
  /** Clear the active thread */
  clearActive: () => void;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useThreadChat(): ThreadChatState {
  const [activeThread, setActiveThread] = useState<Thread | null>(null);
  const [threads, setThreads] = useState<ThreadInfo[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [isThinking, setIsThinking] = useState(false);
  const [streamingText, setStreamingText] = useState("");
  const [contentBlocks, setContentBlocks] = useState<ContentBlock[]>([]);
  const [statusMessage, setStatusMessage] = useState("");
  const [isDone, setIsDone] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const unlistenChunkRef = useRef<(() => void) | null>(null);
  const unlistenResponseRef = useRef<(() => void) | null>(null);
  const unlistenRuntimeErrorRef = useRef<(() => void) | null>(null);
  const unlistenAgentStartRef = useRef<(() => void) | null>(null);

  // Clean up listeners on unmount
  useEffect(() => {
    return () => {
      if (unlistenChunkRef.current) unlistenChunkRef.current();
      if (unlistenResponseRef.current) unlistenResponseRef.current();
      if (unlistenRuntimeErrorRef.current) unlistenRuntimeErrorRef.current();
      if (unlistenAgentStartRef.current) unlistenAgentStartRef.current();
    };
  }, []);

  /** Create a new thread */
  const createNewThread = useCallback(async (agentId: string, _agentName: string): Promise<Thread> => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        // Dev fallback
        const mockThread: Thread = {
          id: `thread-dev-${Date.now()}`,
          agent_id: agentId,
          title: `Thread (dev)`,
          session_id: `session-dev-${Date.now()}`,
          messages: [],
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        setActiveThread(mockThread);
        return mockThread;
      }
      const thread = await createThread(agentId);
      setActiveThread(thread);
      return thread;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /** Load threads list for an agent */
  const loadThreads = useCallback(async (agentId: string) => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        setThreads([]);
        return;
      }
      const list = await listThreads(agentId);
      setThreads(list);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  /** Load all threads across all agents (global list) */
  const loadAllThreads = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        setThreads([]);
        return;
      }
      const list = await listAllThreads();
      setThreads(list);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  /** Select a thread */
  const selectThread = useCallback(async (agentId: string, threadId: string) => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        // Dev fallback
        setActiveThread({
          id: threadId,
          agent_id: agentId,
          title: "Dev Thread",
          session_id: null,
          messages: [],
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        });
        return;
      }
      const thread = await getThread(agentId, threadId);
      setActiveThread(thread);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  /** Delete a thread */
  const removeThread = useCallback(async (agentId: string, threadId: string) => {
    try {
      if (isTauri) {
        await deleteThread(agentId, threadId);
      }
      // If active thread was deleted, clear it
      if (activeThread?.id === threadId) {
        setActiveThread(null);
      }
      // Refresh list
      await loadThreads(agentId);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    }
  }, [activeThread, loadThreads]);

  /** Rename a thread */
  const renameThreadAction = useCallback(async (threadId: string, newTitle: string): Promise<ThreadInfo | null> => {
    try {
      if (!isTauri) return null;
      const updated = await renameThread(threadId, newTitle);
      // Update the thread in the local list
      setThreads((prev) => prev.map((t) => t.id === threadId ? { ...t, title: updated.title, updated_at: updated.updated_at } : t));
      // If this is the active thread, update its title too
      setActiveThread((prev) => {
        if (!prev || prev.id !== threadId) return prev;
        return { ...prev, title: updated.title, updated_at: updated.updated_at };
      });
      return updated;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      return null;
    }
  }, []);

  /** Send a message in a thread */
  const send = useCallback(async (agentId: string, threadId: string, message: string) => {
    setIsThinking(true);
    setIsStreaming(true);
    setIsDone(false);
    setStreamingText("");
    setContentBlocks([]);
    setStatusMessage("");
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
      if (unlistenRuntimeErrorRef.current) {
        unlistenRuntimeErrorRef.current();
        unlistenRuntimeErrorRef.current = null;
      }
      if (unlistenAgentStartRef.current) {
        unlistenAgentStartRef.current();
        unlistenAgentStartRef.current = null;
      }

      if (!isTauri) {
        // Dev fallback: simulate streaming
        setIsThinking(false);

        // Add user message to active thread
        setActiveThread((prev) => {
          if (!prev || prev.id !== threadId) return prev;
          return {
            ...prev,
            messages: [
              ...prev.messages,
              {
                id: `msg-dev-${Date.now()}`,
                role: "user" as const,
                content: message,
                timestamp: new Date().toISOString(),
              },
            ],
          };
        });

        // Simulate streaming text
        const mockResponse = `[Dev Mode] I received your message: "${message}". This is a simulated response.`;
        for (let i = 0; i < mockResponse.length; i++) {
          await new Promise((r) => setTimeout(r, 20));
          setStreamingText(mockResponse.substring(0, i + 1));
        }

        // Add agent message
        setActiveThread((prev) => {
          if (!prev || prev.id !== threadId) return prev;
          return {
            ...prev,
            messages: [
              ...prev.messages,
              {
                id: `msg-dev-${Date.now() + 1}`,
                role: "agent" as const,
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

      // Real Tauri flow:
      // IMPORTANT: Register all listeners BEFORE calling sendMessage IPC.
      // The Rust backend spawns a thread that emits events immediately —
      // if we register listeners after the IPC call returns, we miss events.

      const { listen } = await import("@tauri-apps/api/event");

      let accumulatedText = "";
      let accumulatedBlocks: ContentBlock[] = [];

      // 0. Listen for agent start event to set statusMessage
      const unlistenAgentStart = await listen<{
        thread_id: string;
        agent_id: string;
        runtime_id: string;
        runtime_name: string;
      }>("agent://thread-agent-start", (event) => {
        const payload = event.payload;
        if (payload.thread_id !== threadId) return;
        setStatusMessage(`${payload.runtime_name} initializing...`);
      });
      unlistenAgentStartRef.current = unlistenAgentStart;

      // 1. Listen for streaming chunk events (register BEFORE IPC call)
      const unlistenChunk = await listen<StreamEvent>("agent://chunk", (event) => {
        const payload = event.payload;
        const eventType = payload.type;
        const newBlocks: ContentBlock[] = payload.content_blocks ?? [];

        if (eventType === "assistant" && payload.text) {
          accumulatedText += payload.text;
          setStreamingText(accumulatedText);
          setIsThinking(false); // We received first text chunk

          // Collect content blocks
          if (newBlocks.length > 0) {
            accumulatedBlocks = [...accumulatedBlocks, ...newBlocks];
            setContentBlocks([...accumulatedBlocks]);
          }
          // Clear status message once text starts arriving
          setStatusMessage((prev) => prev && !accumulatedText ? prev : "");

          // Directly update activeThread messages so the agent response
          // appears immediately in the chat (like the reference project).
          setActiveThread((prev) => {
            if (!prev || prev.id !== threadId) return prev;
            const msgs = [...prev.messages];
            const lastMsg = msgs[msgs.length - 1];
            if (lastMsg && lastMsg.role === "agent") {
              // Update existing agent streaming message
              msgs[msgs.length - 1] = { ...lastMsg, content: accumulatedText };
            } else {
              // Add new agent message
              msgs.push({
                id: `stream-${Date.now()}`,
                role: "agent" as const,
                content: accumulatedText,
                timestamp: new Date().toISOString(),
              });
            }
            return { ...prev, messages: msgs };
          });
        } else if (eventType === "assistant" && newBlocks.length > 0) {
          // Assistant event with content_blocks but no text (e.g. tool_use during thinking)
          accumulatedBlocks = [...accumulatedBlocks, ...newBlocks];
          setContentBlocks([...accumulatedBlocks]);
        } else if (eventType === "user" && newBlocks.length > 0) {
          // User events carry tool_result blocks
          accumulatedBlocks = [...accumulatedBlocks, ...newBlocks];
          setContentBlocks([...accumulatedBlocks]);
        } else if (eventType === "system" && payload.text) {
          // System events carry initialization status
          setStatusMessage(payload.text);
        }

        if (payload.is_done) {
          setIsStreaming(false);
          setIsThinking(false);
          setIsDone(true);
          // Clear contentBlocks on done (they were ephemeral)
          setContentBlocks([]);
          setStatusMessage("");
        }
      });
      unlistenChunkRef.current = unlistenChunk;

      // 2. Listen for runtime unavailability events
      const unlistenRuntimeError = await listen<{
        agent_id: string;
        thread_id: string;
        runtime_id: string;
        runtime_name: string;
        install_hint: string;
        error: string;
      }>("runtime://unavailable", async (event) => {
        const { error: runtimeError, runtime_name, install_hint } = event.payload;
        setError(`${runtime_name}: ${runtimeError}${install_hint ? `\nInstall: ${install_hint}` : ""}`);
        setIsStreaming(false);
        setIsThinking(false);
        setIsDone(false);
        setContentBlocks([]);
        setStatusMessage("");
        unlistenRuntimeErrorRef.current = null;
      });
      unlistenRuntimeErrorRef.current = unlistenRuntimeError;

      // 3. Listen for the thread-response event to persist the agent response
      const unlistenResponse = await listen<{
        thread_id: string;
        agent_id: string;
        content: string;
        session_id: string | null;
      }>("agent://thread-response", async (event) => {
        const { content, session_id } = event.payload;

        // Save agent response to backend and reload thread with final state
        try {
          const finalThread = await saveAgentResponse(agentId, threadId, content, session_id);
          setActiveThread(finalThread);
        } catch (err) {
          console.error("[useThreadChat] save_agent_response failed:", err);
        }

        setStreamingText("");

        // Clean up listeners
        unlistenChunk();
        unlistenResponse();
        unlistenRuntimeError();
        unlistenAgentStart();
        unlistenChunkRef.current = null;
        unlistenResponseRef.current = null;
        unlistenRuntimeErrorRef.current = null;
        unlistenAgentStartRef.current = null;
      });
      unlistenResponseRef.current = unlistenResponse;

      // 4. NOW call sendMessage IPC — listeners are already registered
      const updatedThread = await sendMessage(agentId, threadId, message);
      setActiveThread(updatedThread);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      setIsStreaming(false);
      setIsThinking(false);
    }
  }, []);

  /** Clear the active thread */
  const clearActive = useCallback(() => {
    setActiveThread(null);
    setStreamingText("");
    setIsStreaming(false);
    setIsThinking(false);
    setIsDone(false);
    setContentBlocks([]);
    setStatusMessage("");
  }, []);

  return {
    activeThread,
    threads,
    isStreaming,
    isThinking,
    streamingText,
    contentBlocks,
    statusMessage,
    isDone,
    loading,
    error,
    createNewThread,
    loadThreads,
    loadAllThreads,
    selectThread,
    removeThread,
    renameThreadAction,
    send,
    clearActive,
  };
}
