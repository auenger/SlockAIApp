/**
 * Hook for computing aggregated token usage statistics for an agent.
 *
 * Scans available messages (channel + thread) for the agent and accumulates
 * token_usage by model. Returns a summary suitable for display in the
 * Agent Profile page.
 */

import { useMemo } from 'react';
import type { TokenUsage } from '../types';

/** Aggregated token usage for a single model */
export interface ModelTokenSummary {
  model: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_write_tokens: number;
  total_tokens: number;
}

/** Aggregated token usage across all models for an agent */
export interface TokenUsageSummary {
  models: ModelTokenSummary[];
  grandTotal: number;
}

/**
 * Aggregate token usage records by model name.
 * Accepts an array of token_usage maps (one per message) and merges them.
 */
export function aggregateTokenUsage(
  usageRecords: Array<Record<string, TokenUsage> | undefined>
): TokenUsageSummary {
  const acc = new Map<string, TokenUsage>();

  for (const usage of usageRecords) {
    if (!usage) continue;
    for (const [model, tokens] of Object.entries(usage)) {
      const existing = acc.get(model);
      if (existing) {
        existing.input_tokens += tokens.input_tokens;
        existing.output_tokens += tokens.output_tokens;
        existing.cache_read_tokens += tokens.cache_read_tokens;
        existing.cache_write_tokens += tokens.cache_write_tokens;
      } else {
        acc.set(model, { ...tokens });
      }
    }
  }

  let grandTotal = 0;
  const models: ModelTokenSummary[] = [];

  for (const [model, tokens] of acc) {
    const total = tokens.input_tokens + tokens.output_tokens + tokens.cache_read_tokens + tokens.cache_write_tokens;
    grandTotal += total;
    models.push({
      model,
      ...tokens,
      total_tokens: total,
    });
  }

  // Sort by total tokens descending
  models.sort((a, b) => b.total_tokens - a.total_tokens);

  return { models, grandTotal };
}

/**
 * Hook to compute aggregated token usage for an agent from its messages.
 *
 * @param messages - Array of messages that may contain token_usage (only agent messages count)
 * @returns TokenUsageSummary
 */
export function useTokenUsageSummary(
  messages: Array<{ token_usage?: Record<string, TokenUsage> }>
): TokenUsageSummary {
  return useMemo(() => {
    const usageRecords = messages
      .filter((m) => m.token_usage)
      .map((m) => m.token_usage);
    return aggregateTokenUsage(usageRecords);
  }, [messages]);
}
