/**
 * TokenUsageBadge — Displays token usage statistics for an agent message.
 *
 * Two states:
 * - Collapsed (default): "1.2k tokens" inline badge
 * - Expanded (hover/click): breakdown by model with input/output/cache distribution
 *
 * Only renders when token_usage data is present.
 */

import React, { useState } from 'react';
import { cn } from '../lib/utils';
import type { TokenUsage } from '../types';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Format a token count for compact display: >1000 → "1.2k", >1M → "1.2M" */
function formatTokenCount(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}k`;
  }
  return String(count);
}

/** Compute total tokens across all models in a token_usage map */
function totalTokens(tokenUsage: Record<string, TokenUsage>): number {
  let total = 0;
  for (const usage of Object.values(tokenUsage)) {
    total += usage.input_tokens + usage.output_tokens + usage.cache_read_tokens + usage.cache_write_tokens;
  }
  return total;
}

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TokenUsageBadgeProps {
  /** Token usage by model name */
  tokenUsage: Record<string, TokenUsage>;
  /** Additional CSS class */
  className?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TokenUsageBadge: React.FC<TokenUsageBadgeProps> = ({ tokenUsage, className }) => {
  const [expanded, setExpanded] = useState(false);

  const total = totalTokens(tokenUsage);
  if (total === 0) return null;

  const models = Object.entries(tokenUsage);
  const hasMultipleModels = models.length > 1;

  return (
    <div
      className={cn("inline-block", className)}
      onMouseEnter={() => setExpanded(true)}
      onMouseLeave={() => setExpanded(false)}
    >
      {/* Collapsed badge */}
      {!expanded && (
        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 text-[9px] font-mono text-gray-500 bg-gray-100 brutal-border cursor-default select-none">
          <svg width="8" height="8" viewBox="0 0 16 16" fill="none" className="text-gray-400">
            <circle cx="8" cy="8" r="7" stroke="currentColor" strokeWidth="1.5" />
            <path d="M8 4v4l3 2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
          {formatTokenCount(total)} tokens
        </span>
      )}

      {/* Expanded breakdown */}
      {expanded && (
        <div className="brutal-border bg-white brutal-shadow p-2 text-[10px] font-mono min-w-[180px] z-50 relative">
          <div className="font-black uppercase text-[8px] text-gray-400 mb-1.5 tracking-wider">Token Usage</div>
          {models.map(([model, usage]) => (
            <div key={model} className="mb-1.5 last:mb-0">
              {hasMultipleModels && (
                <div className="font-bold text-[9px] text-brutal-cyan mb-0.5 truncate" title={model}>
                  {model}
                </div>
              )}
              <div className="grid grid-cols-2 gap-x-3 gap-y-0.5 text-gray-600">
                <span className="text-gray-400">input</span>
                <span className="text-right">{formatTokenCount(usage.input_tokens)}</span>
                <span className="text-gray-400">output</span>
                <span className="text-right">{formatTokenCount(usage.output_tokens)}</span>
                {usage.cache_read_tokens > 0 && (
                  <>
                    <span className="text-gray-400">cache read</span>
                    <span className="text-right">{formatTokenCount(usage.cache_read_tokens)}</span>
                  </>
                )}
                {usage.cache_write_tokens > 0 && (
                  <>
                    <span className="text-gray-400">cache write</span>
                    <span className="text-right">{formatTokenCount(usage.cache_write_tokens)}</span>
                  </>
                )}
              </div>
            </div>
          ))}
          <div className="mt-1.5 pt-1 brutal-border-t flex items-center justify-between font-bold">
            <span>Total</span>
            <span>{formatTokenCount(total)}</span>
          </div>
        </div>
      )}
    </div>
  );
};
