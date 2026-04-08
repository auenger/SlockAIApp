/**
 * Hook for managing agent runtimes in AgentsZone.
 *
 * Provides runtime scanning, listing, session management,
 * and streaming message execution via Tauri IPC.
 */

import { useState, useCallback, useRef } from "react";
import type {
  AgentRuntimeInfo,
  StreamEvent,
} from "../types";
import { invoke } from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback: mock data when not running inside Tauri
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

const MOCK_RUNTIMES: AgentRuntimeInfo[] = [
  {
    id: "claude-code",
    name: "Claude Code",
    runtime_type: "cli",
    status: "available",
    version: "1.0.0-dev",
    install_path: "/usr/local/bin/claude",
    capabilities: ["streaming", "sessions", "tool_use", "structured_output"],
    install_hint: "npm install -g @anthropic-ai/claude-code",
  },
];

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface AgentRuntimeState {
  /** List of discovered runtimes */
  runtimes: AgentRuntimeInfo[];
  /** Whether a scan is in progress */
  scanning: boolean;
  /** Current active session ID */
  sessionId: string | null;
  /** Buffered stream events from the last execution */
  streamEvents: StreamEvent[];
  /** Scan for available runtimes */
  scan: () => Promise<void>;
  /** Start a new session on a runtime */
  startSession: (runtimeId: string) => Promise<string>;
  /** Stop the current session */
  stopSession: () => Promise<void>;
  /** Execute a message on a runtime and collect stream events */
  execute: (
    runtimeId: string,
    message: string,
    sessionId?: string,
    systemPrompt?: string
  ) => Promise<void>;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useAgentRuntimes(): AgentRuntimeState {
  const [runtimes, setRuntimes] = useState<AgentRuntimeInfo[]>([]);
  const [scanning, setScanning] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [streamEvents, setStreamEvents] = useState<StreamEvent[]>([]);
  const unlistenRef = useRef<(() => void) | null>(null);

  /** Scan for available runtimes */
  const scan = useCallback(async () => {
    setScanning(true);
    try {
      if (!isTauri) {
        setRuntimes(MOCK_RUNTIMES);
        return;
      }
      const result = await invoke<AgentRuntimeInfo[]>("scan_agent_runtimes");
      setRuntimes(result);
    } catch (err) {
      console.error("[useAgentRuntimes] scan failed:", err);
      if (!isTauri) {
        setRuntimes(MOCK_RUNTIMES);
      }
    } finally {
      setScanning(false);
    }
  }, []);

  /** Start a new session on a runtime */
  const startSession = useCallback(async (runtimeId: string): Promise<string> => {
    if (!isTauri) {
      const mockId = `session-dev-${Date.now()}`;
      setSessionId(mockId);
      return mockId;
    }
    const sid = await invoke<string>("runtime_session_start", { runtimeId });
    setSessionId(sid);
    return sid;
  }, []);

  /** Stop the current session */
  const stopSession = useCallback(async () => {
    try {
      if (isTauri) {
        await invoke("runtime_session_stop");
      }
    } finally {
      setSessionId(null);
    }
  }, []);

  /** Execute a message on a runtime */
  const execute = useCallback(
    async (
      runtimeId: string,
      message: string,
      sid?: string,
      systemPrompt?: string
    ) => {
      setStreamEvents([]);

      // Clean up previous listener
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }

      if (!isTauri) {
        // Dev fallback: simulate a streaming response
        const mockEvents: StreamEvent[] = [
          { text: "Hello! I am Claude Code", is_done: false, type: "assistant" },
          { text: " running in dev mode.", is_done: false, type: "assistant" },
          { text: "", is_done: true, type: "result", session_id: sid ?? undefined },
        ];
        for (const event of mockEvents) {
          setStreamEvents((prev) => [...prev, event]);
          await new Promise((r) => setTimeout(r, 300));
        }
        return;
      }

      // Listen for streaming events from the backend
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<StreamEvent>("agent://chunk", (event) => {
        setStreamEvents((prev) => [...prev, event.payload]);
      });
      unlistenRef.current = unlisten;

      try {
        await invoke("runtime_execute", {
          runtimeId,
          message,
          sessionId: sid ?? null,
          systemPrompt: systemPrompt ?? null,
        });
      } finally {
        // Clean up listener after execution completes
        // Note: we delay slightly to catch any final events
        setTimeout(() => {
          unlisten();
          unlistenRef.current = null;
        }, 500);
      }
    },
    []
  );

  return {
    runtimes,
    scanning,
    sessionId,
    streamEvents,
    scan,
    startSession,
    stopSession,
    execute,
  };
}
