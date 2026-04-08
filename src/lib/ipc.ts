/**
 * Type-safe Tauri IPC wrapper.
 *
 * Provides invoke and listen helpers with proper TypeScript typing
 * for communication between the React frontend and Rust backend.
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/**
 * Type-safe invoke wrapper for Tauri commands.
 */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}

/**
 * Send a greeting to the Rust backend (test command).
 */
export async function greet(name: string): Promise<string> {
  return invoke<string>("greet", { name });
}
