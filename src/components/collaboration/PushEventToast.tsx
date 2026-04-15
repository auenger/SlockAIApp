/**
 * PushEventToast — Toast notification for push events from remote agents.
 *
 * Shows a brief notification when a task completes, fails, or needs input.
 * Auto-dismisses after a timeout.
 */

import { useEffect, useState } from "react";

interface PushEventToastProps {
  event: {
    event_type: string;
    agent_id: string;
    task_id: string;
    message: string;
    timestamp: string;
  };
  duration?: number;
  onDismiss?: () => void;
}

const EVENT_ICONS: Record<string, string> = {
  task_completed: "\u2713",
  task_failed: "\u2717",
  input_required: "?",
  task_updated: "\u2192",
  artifact_available: "\u25C6",
};

const EVENT_COLORS: Record<string, string> = {
  task_completed: "border-green-500/50 bg-green-500/10",
  task_failed: "border-red-500/50 bg-red-500/10",
  input_required: "border-yellow-500/50 bg-yellow-500/10",
  task_updated: "border-blue-500/50 bg-blue-500/10",
  artifact_available: "border-purple-500/50 bg-purple-500/10",
};

export function PushEventToast({ event, duration = 5000, onDismiss }: PushEventToastProps) {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => {
      setVisible(false);
      onDismiss?.();
    }, duration);

    return () => clearTimeout(timer);
  }, [duration, onDismiss]);

  if (!visible) return null;

  const icon = EVENT_ICONS[event.event_type] || "\u2022";
  const colors = EVENT_COLORS[event.event_type] || "border-gray-500/50 bg-gray-500/10";

  return (
    <div
      className={`border rounded-lg px-3 py-2 shadow-lg ${colors} transition-opacity`}
    >
      <div className="flex items-center gap-2">
        <span className="text-sm">{icon}</span>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium text-gray-200">
              {event.agent_id}
            </span>
            <span className="text-[10px] text-gray-500">
              {event.event_type.replace(/_/g, " ")}
            </span>
          </div>
          {event.message && (
            <p className="text-[11px] text-gray-400 truncate mt-0.5">
              {event.message}
            </p>
          )}
        </div>
        <button
          onClick={() => {
            setVisible(false);
            onDismiss?.();
          }}
          className="text-gray-500 hover:text-gray-300 text-xs"
        >
          \u2715
        </button>
      </div>
    </div>
  );
}

export default PushEventToast;
