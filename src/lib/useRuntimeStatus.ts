/**
 * Hook for runtime status detection in the Create Agent modal.
 *
 * Provides a list of all known runtimes with their availability status.
 * Triggers a scan when requested and caches the results.
 */

import { useState, useCallback } from "react";
import type { AgentRuntimeInfo } from "../types";
import { scanAgentRuntimes, listAgentRuntimes } from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback: mock data when not running inside Tauri
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

const MOCK_RUNTIMES: AgentRuntimeInfo[] = [
  {
    id: "claude-code",
    name: "Claude Code",
    runtime_category: "cli",
    runtime_type: "claude_code",
    status: "available",
    version: "1.0.3",
    install_path: "/usr/local/bin/claude",
    capabilities: ["streaming", "sessions", "tool_use", "structured_output"],
    install_hint: "npm install -g @anthropic-ai/claude-code",
    binary_name: "claude",
  },
  {
    id: "codex",
    name: "Codex",
    runtime_category: "cli",
    runtime_type: "codex",
    status: "not-installed",
    install_hint: "npm install -g @openai/codex",
    capabilities: ["streaming", "tool_use"],
    binary_name: "codex",
  },
  {
    id: "gemini",
    name: "Gemini CLI",
    runtime_category: "cli",
    runtime_type: "gemini",
    status: "not-installed",
    install_hint: "npm install -g @anthropic-ai/gemini-cli",
    capabilities: ["streaming"],
    binary_name: "gemini",
  },
];

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface RuntimeStatusState {
  /** List of runtimes with their availability status */
  runtimes: AgentRuntimeInfo[];
  /** Whether a scan is in progress */
  scanning: boolean;
  /** Trigger a fresh scan for runtimes */
  refresh: () => Promise<void>;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useRuntimeStatus(): RuntimeStatusState {
  const [runtimes, setRuntimes] = useState<AgentRuntimeInfo[]>([]);
  const [scanning, setScanning] = useState(false);

  /** Trigger a runtime scan and update the list */
  const refresh = useCallback(async () => {
    setScanning(true);
    try {
      if (!isTauri) {
        setRuntimes(MOCK_RUNTIMES);
        return;
      }

      // First scan to detect what's installed
      await scanAgentRuntimes();

      // Then get the full list
      const result = await listAgentRuntimes();
      setRuntimes(result);
    } catch (err) {
      console.error("[useRuntimeStatus] refresh failed:", err);
      if (!isTauri) {
        setRuntimes(MOCK_RUNTIMES);
      }
    } finally {
      setScanning(false);
    }
  }, []);

  return { runtimes, scanning, refresh };
}
