/**
 * AgentTaskCard — Displays a single agent's delegation status in the collaboration view.
 *
 * Shows:
 * - Agent name and task description
 * - Status badge with color coding
 * - Progress indication
 * - Action buttons (cancel, retry)
 */

// No React import needed for JSX transform

interface AgentTaskCardProps {
  delegation: {
    id: string;
    from_agent_id: string;
    to_agent_id: string;
    task_description: string;
    status: string;
    result: string | null;
    error: string | null;
    created_at: string;
  };
  onCancel: () => void;
  onRetry: () => void;
}

const STATUS_STYLES: Record<string, { bg: string; text: string; dot: string }> = {
  PENDING: { bg: "bg-gray-500/20", text: "text-gray-400", dot: "bg-gray-400" },
  SENT: { bg: "bg-yellow-500/20", text: "text-yellow-400", dot: "bg-yellow-400" },
  ACKNOWLEDGED: { bg: "bg-blue-500/20", text: "text-blue-400", dot: "bg-blue-400" },
  IN_PROGRESS: { bg: "bg-blue-500/20", text: "text-blue-400", dot: "bg-blue-400" },
  COMPLETED: { bg: "bg-green-500/20", text: "text-green-400", dot: "bg-green-400" },
  FAILED: { bg: "bg-red-500/20", text: "text-red-400", dot: "bg-red-400" },
  CANCELLED: { bg: "bg-gray-500/20", text: "text-gray-500", dot: "bg-gray-500" },
  TIMED_OUT: { bg: "bg-orange-500/20", text: "text-orange-400", dot: "bg-orange-400" },
};

function StatusBadge({ status }: { status: string }) {
  const style = STATUS_STYLES[status] || STATUS_STYLES.PENDING;
  return (
    <span className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-medium ${style.bg} ${style.text}`}>
      <span className={`w-1.5 h-1.5 rounded-full ${style.dot} ${status === "IN_PROGRESS" ? "animate-pulse" : ""}`} />
      {status.replace("_", " ")}
    </span>
  );
}

export function AgentTaskCard({ delegation, onCancel, onRetry }: AgentTaskCardProps) {
  const isTerminal = ["COMPLETED", "FAILED", "CANCELLED", "TIMED_OUT"].includes(delegation.status);
  const canCancel = !isTerminal && delegation.status !== "COMPLETED";
  const canRetry = delegation.status === "FAILED" || delegation.status === "TIMED_OUT";

  return (
    <div className="bg-gray-800/50 border border-gray-700/50 rounded-lg p-3 space-y-2">
      {/* Header: agent arrow + status */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 text-xs text-gray-300">
          <span className="font-medium">{delegation.from_agent_id}</span>
          <span className="text-gray-600">→</span>
          <span className="font-medium">{delegation.to_agent_id}</span>
        </div>
        <StatusBadge status={delegation.status} />
      </div>

      {/* Task description */}
      <p className="text-xs text-gray-400 line-clamp-2">
        {delegation.task_description}
      </p>

      {/* Result or error */}
      {delegation.result && (
        <div className="text-xs text-green-400/80 bg-green-500/10 rounded px-2 py-1 line-clamp-3">
          {delegation.result}
        </div>
      )}
      {delegation.error && (
        <div className="text-xs text-red-400/80 bg-red-500/10 rounded px-2 py-1">
          Error: {delegation.error}
        </div>
      )}

      {/* Actions */}
      {!isTerminal && (
        <div className="flex items-center gap-2 pt-1">
          {canCancel && (
            <button
              onClick={onCancel}
              className="text-[10px] text-gray-500 hover:text-red-400 transition-colors"
            >
              Cancel
            </button>
          )}
          {delegation.status === "IN_PROGRESS" && (
            <span className="text-[10px] text-blue-400 animate-pulse">Working...</span>
          )}
        </div>
      )}

      {canRetry && (
        <button
          onClick={onRetry}
          className="text-[10px] text-blue-400 hover:text-blue-300 transition-colors"
        >
          Retry
        </button>
      )}

      {/* Timestamp */}
      <div className="text-[10px] text-gray-600">
        {new Date(delegation.created_at).toLocaleString()}
      </div>
    </div>
  );
}

export default AgentTaskCard;
