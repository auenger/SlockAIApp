/**
 * Hook for workspace file browser in AgentsZone.
 *
 * Provides directory listing and file content loading for agent workspaces.
 */

import { useState, useCallback } from "react";
import type { DirectoryEntry, FileContent } from "../types";
import { listWorkspaceDir, readWorkspaceFile } from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback: mock data when not running inside Tauri
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

const MOCK_ENTRIES: DirectoryEntry[] = [
  { name: "conversations", is_dir: true, size: 0, modified: Date.now() / 1000 },
  { name: "context", is_dir: true, size: 0, modified: Date.now() / 1000 },
  { name: "output", is_dir: true, size: 0, modified: Date.now() / 1000 },
  { name: "skills", is_dir: true, size: 0, modified: Date.now() / 1000 },
  { name: "config", is_dir: true, size: 0, modified: Date.now() / 1000 },
  { name: "IDENTITY.md", is_dir: false, size: 256, modified: Date.now() / 1000 - 86400 },
  { name: "SOUL.md", is_dir: false, size: 1024, modified: Date.now() / 1000 - 86400 },
  { name: "MEMORY.md", is_dir: false, size: 2048, modified: Date.now() / 1000 - 3600 },
];

const MOCK_FILE_CONTENT: FileContent = {
  path: "/workspace/default/MEMORY.md",
  name: "MEMORY.md",
  size: 2048,
  mime_type: "text/markdown",
  content: `# Agent Memory

This file contains persistent context for the agent.

## Current Context

- Project: KAgent Integration
- Platform: SAP AI Core
- Status: Architecture Review

## Runtime Info

\`\`\`
last_sync: 2026-04-07T02:30:00Z
active_threads: 2
\`\`\`
`,
};

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface WorkspaceState {
  /** Current directory entries */
  entries: DirectoryEntry[];
  /** Currently selected file content */
  selectedFile: FileContent | null;
  /** Current workspace path */
  workspacePath: string;
  /** Current subpath within workspace */
  currentPath: string;
  /** Loading state for directory */
  loadingDir: boolean;
  /** Loading state for file */
  loadingFile: boolean;
  /** Error message if any */
  error: string | null;
  /** Load directory contents */
  loadDir: (agentId: string, subpath?: string) => Promise<void>;
  /** Load file content */
  loadFile: (agentId: string, filePath: string) => Promise<void>;
  /** Navigate into a directory */
  navigateInto: (agentId: string, dirName: string) => Promise<void>;
  /** Navigate up one directory */
  navigateUp: (agentId: string) => Promise<void>;
  /** Clear selected file */
  clearSelectedFile: () => void;
  /** Reset workspace state */
  reset: () => void;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useWorkspace(): WorkspaceState {
  const [entries, setEntries] = useState<DirectoryEntry[]>([]);
  const [selectedFile, setSelectedFile] = useState<FileContent | null>(null);
  const [currentPath, setCurrentPath] = useState<string>("");
  const [workspacePath, setWorkspacePath] = useState<string>("");
  const [loadingDir, setLoadingDir] = useState(false);
  const [loadingFile, setLoadingFile] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /** Load directory contents */
  const loadDir = useCallback(async (agentId: string, subpath?: string) => {
    setLoadingDir(true);
    setError(null);
    try {
      if (!isTauri) {
        setEntries(MOCK_ENTRIES);
        setWorkspacePath(`~/.agentszone/agents/${agentId}/`);
        setCurrentPath(subpath || "");
        return;
      }

      const result = await listWorkspaceDir(agentId, subpath);
      setEntries(result);
      setWorkspacePath(`~/.agentszone/agents/${agentId}/`);
      setCurrentPath(subpath || "");
    } catch (err) {
      console.error("[useWorkspace] loadDir failed:", err);
      setError(String(err));
      setEntries([]);
    } finally {
      setLoadingDir(false);
    }
  }, []);

  /** Load file content */
  const loadFile = useCallback(async (agentId: string, filePath: string) => {
    setLoadingFile(true);
    setError(null);
    try {
      if (!isTauri) {
        setSelectedFile(MOCK_FILE_CONTENT);
        return;
      }

      const result = await readWorkspaceFile(agentId, filePath);
      setSelectedFile(result);
    } catch (err) {
      console.error("[useWorkspace] loadFile failed:", err);
      setError(String(err));
      setSelectedFile(null);
    } finally {
      setLoadingFile(false);
    }
  }, []);

  /** Navigate into a subdirectory */
  const navigateInto = useCallback(
    async (agentId: string, dirName: string) => {
      const newPath = currentPath ? `${currentPath}/${dirName}` : dirName;
      await loadDir(agentId, newPath);
      setSelectedFile(null);
    },
    [currentPath, loadDir]
  );

  /** Navigate up one directory */
  const navigateUp = useCallback(
    async (agentId: string) => {
      if (!currentPath) return;
      const parts = currentPath.split("/");
      parts.pop();
      const parentPath = parts.length > 0 ? parts.join("/") : "";
      await loadDir(agentId, parentPath || undefined);
      setSelectedFile(null);
    },
    [currentPath, loadDir]
  );

  /** Clear selected file */
  const clearSelectedFile = useCallback(() => {
    setSelectedFile(null);
  }, []);

  /** Reset workspace state */
  const reset = useCallback(() => {
    setEntries([]);
    setSelectedFile(null);
    setCurrentPath("");
    setWorkspacePath("");
    setError(null);
  }, []);

  return {
    entries,
    selectedFile,
    workspacePath,
    currentPath,
    loadingDir,
    loadingFile,
    error,
    loadDir,
    loadFile,
    navigateInto,
    navigateUp,
    clearSelectedFile,
    reset,
  };
}
