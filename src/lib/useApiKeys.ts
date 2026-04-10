/**
 * Hook for API Key management in AgentsZone.
 *
 * Provides listing, adding, deleting, and verifying API keys
 * stored in the OS keyring via the Rust backend.
 */

import { useState, useCallback } from "react";
import type { ApiKeyInfo } from "../types";
import { listApiKeys, storeApiKey, deleteApiKey, verifyApiKey } from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback: mock data when not running inside Tauri
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

const MOCK_KEYS: ApiKeyInfo[] = [
  { id: "claude-code", name: "Claude Code", masked_key: "sk-***...a3f", has_key: true },
  { id: "openai", name: "OpenAI", masked_key: "", has_key: false },
  { id: "anthropic", name: "Anthropic", masked_key: "sk-***...7b2", has_key: true },
  { id: "gemini", name: "Gemini", masked_key: "", has_key: false },
];

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface ApiKeysState {
  /** List of API key info (masked) */
  keys: ApiKeyInfo[];
  /** Loading state */
  loading: boolean;
  /** Error message if any */
  error: string | null;
  /** Load all API keys */
  loadKeys: () => Promise<void>;
  /** Add a new API key */
  addKey: (runtimeId: string, apiKey: string) => Promise<void>;
  /** Delete an API key */
  removeKey: (runtimeId: string) => Promise<void>;
  /** Verify a single API key */
  verify: (runtimeId: string) => Promise<ApiKeyInfo | null>;
  /** Clear error */
  clearError: () => void;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useApiKeys(): ApiKeysState {
  const [keys, setKeys] = useState<ApiKeyInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /** Load all API keys from backend */
  const loadKeys = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      if (!isTauri) {
        setKeys(MOCK_KEYS);
        return;
      }
      const result = await listApiKeys();
      setKeys(result);
    } catch (err) {
      console.error("[useApiKeys] loadKeys failed:", err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  /** Add (store) a new API key */
  const addKey = useCallback(async (runtimeId: string, apiKey: string) => {
    setError(null);
    try {
      if (!isTauri) {
        // Mock: update local state
        setKeys((prev) =>
          prev.map((k) =>
            k.id === runtimeId
              ? { ...k, masked_key: `sk-***...${apiKey.slice(-3)}`, has_key: true }
              : k
          )
        );
        return;
      }
      await storeApiKey(runtimeId, apiKey);
      // Reload keys after adding
      await loadKeys();
    } catch (err) {
      console.error("[useApiKeys] addKey failed:", err);
      setError(String(err));
      throw err;
    }
  }, [loadKeys]);

  /** Delete an API key */
  const removeKey = useCallback(async (runtimeId: string) => {
    setError(null);
    try {
      if (!isTauri) {
        // Mock: update local state
        setKeys((prev) =>
          prev.map((k) =>
            k.id === runtimeId
              ? { ...k, masked_key: "", has_key: false }
              : k
          )
        );
        return;
      }
      await deleteApiKey(runtimeId);
      // Reload keys after deletion
      await loadKeys();
    } catch (err) {
      console.error("[useApiKeys] removeKey failed:", err);
      setError(String(err));
      throw err;
    }
  }, [loadKeys]);

  /** Verify a single API key */
  const verify = useCallback(async (runtimeId: string): Promise<ApiKeyInfo | null> => {
    setError(null);
    try {
      if (!isTauri) {
        return MOCK_KEYS.find((k) => k.id === runtimeId) || null;
      }
      const result = await verifyApiKey(runtimeId);
      return result;
    } catch (err) {
      console.error("[useApiKeys] verify failed:", err);
      setError(String(err));
      return null;
    }
  }, []);

  /** Clear error state */
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    keys,
    loading,
    error,
    loadKeys,
    addKey,
    removeKey,
    verify,
    clearError,
  };
}
