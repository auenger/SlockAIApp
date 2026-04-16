/**
 * LanAccessPanel -- UI for enabling/disabling LAN A2A server access.
 *
 * Provides a toggle switch, port configuration, status indicator,
 * local IP display, and copy-to-clipboard for connection URLs.
 */

import { useState } from "react";
import { useLanServer } from "../../lib/useLanServer";

// ===========================================================================
// Status indicator
// ===========================================================================

function StatusIndicator({ status }: { status: string | { error: string } }) {
  const config: Record<string, { color: string; label: string }> = {
    running: { color: "bg-green-500", label: "Running" },
    stopped: { color: "bg-gray-400", label: "Stopped" },
  };

  // Handle error status (object instead of string)
  const statusStr = typeof status === "string" ? status : "error";
  if (statusStr === "error") {
    return (
      <span className="inline-flex items-center gap-1.5 text-xs text-red-400">
        <span className="w-2 h-2 rounded-full bg-red-500" />
        Error
      </span>
    );
  }

  const { color, label } = config[statusStr] ?? {
    color: "bg-gray-400",
    label: statusStr,
  };

  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-gray-500">
      <span className={`w-2 h-2 rounded-full ${color}`} />
      {label}
    </span>
  );
}

// ===========================================================================
// Main panel
// ===========================================================================

export default function LanAccessPanel() {
  const { serverInfo, isRunning, loading, error, start, stop } =
    useLanServer();
  const [port, setPort] = useState(7878);
  const [copied, setCopied] = useState<string | null>(null);

  const handleToggle = async () => {
    if (isRunning) {
      await stop();
    } else {
      await start(port);
    }
  };

  const handleCopy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(text);
      setTimeout(() => setCopied(null), 2000);
    } catch {
      // Fallback: no clipboard access
    }
  };

  const ips = serverInfo?.local_ips ?? [];
  const currentPort = serverInfo?.port ?? port;

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-white">LAN Access</h3>
        {serverInfo && <StatusIndicator status={serverInfo.status} />}
      </div>

      {/* Description */}
      <p className="text-xs text-gray-400">
        Allow other devices on your local network to connect to this AgentsZone
        instance via the A2A protocol.
      </p>

      {/* Error display */}
      {error && (
        <div className="text-xs text-red-400 bg-red-900/20 rounded px-3 py-2">
          {error}
        </div>
      )}

      {/* Toggle + Port config */}
      <div className="flex items-center gap-4">
        {/* Toggle switch */}
        <button
          onClick={handleToggle}
          disabled={loading}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
            isRunning ? "bg-green-500" : "bg-gray-600"
          } ${loading ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              isRunning ? "translate-x-6" : "translate-x-1"
            }`}
          />
        </button>

        {/* Port input */}
        <div className="flex items-center gap-2">
          <label className="text-xs text-gray-400">Port:</label>
          <input
            type="number"
            value={port}
            onChange={(e) => setPort(Number(e.target.value))}
            disabled={isRunning}
            min={1024}
            max={65535}
            className="w-20 bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-white text-center focus:outline-none focus:border-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          />
        </div>
      </div>

      {/* Connection info (shown when running) */}
      {isRunning && ips.length > 0 && (
        <div className="space-y-2">
          <h4 className="text-xs font-medium text-gray-400">
            Connection Addresses
          </h4>
          <div className="space-y-1.5">
            {ips.map((ip) => {
              const url = `http://${ip}:${currentPort}/a2a`;
              const agentCardUrl = `http://${ip}:${currentPort}/agent-card`;
              return (
                <div
                  key={ip}
                  className="flex items-center justify-between bg-gray-800/50 border border-gray-700 rounded px-3 py-2"
                >
                  <div>
                    <span className="text-xs font-mono text-gray-300">
                      {url}
                    </span>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() => handleCopy(url)}
                      className="px-2 py-0.5 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600"
                      title="Copy A2A endpoint URL"
                    >
                      {copied === url ? "Copied!" : "Copy"}
                    </button>
                    <button
                      onClick={() => handleCopy(agentCardUrl)}
                      className="px-2 py-0.5 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600"
                      title="Copy Agent Card URL"
                    >
                      {copied === agentCardUrl ? "Copied!" : "Card"}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
          <p className="text-[10px] text-gray-500">
            Share the &quot;/a2a&quot; URL with other AgentsZone instances to
            enable A2A communication. Use the &quot;Card&quot; URL to test
            connectivity.
          </p>
        </div>
      )}

      {/* Stopped state info */}
      {!isRunning && (
        <div className="text-center py-4 text-gray-500 text-xs">
          LAN access is disabled. Toggle the switch above to enable.
        </div>
      )}
    </div>
  );
}
