/**
 * CollaborationView — Displays A2A multi-agent collaboration status in a Channel.
 *
 * Shows:
 * - Active delegations between agents
 * - Agent task cards with status
 * - Push notification events
 * - Artifact browser
 */

import { useState } from "react";
import { useCollaboration, usePushEvents, useArtifacts } from "../../lib/useCollaboration";
import { AgentTaskCard } from "./AgentTaskCard";
import { PushEventToast } from "./PushEventToast";

interface CollaborationViewProps {
  channelId: string;
  agentIds: string[];
}

export function CollaborationView({ channelId }: CollaborationViewProps) {
  const [activeTab, setActiveTab] = useState<"delegations" | "artifacts" | "events">("delegations");
  const { delegations, activeDelegations, loading: delegationsLoading, cancel: cancelDelegation, retry } = useCollaboration();
  const { events, configs, registerPushUrl, clearEvents } = usePushEvents();
  const { artifacts, loading: artifactsLoading } = useArtifacts();
  const [showPushConfig, setShowPushConfig] = useState(false);
  const [pushUrl, setPushUrl] = useState("http://localhost:9470/push");

  const channelDelegations = delegations.filter((d) => d.channel_id === channelId);
  const channelActiveDelegations = activeDelegations.filter((d) => d.channel_id === channelId);

  const tabs = [
    { key: "delegations" as const, label: "Delegations", count: channelActiveDelegations.length },
    { key: "artifacts" as const, label: "Artifacts", count: artifacts.length },
    { key: "events" as const, label: "Events", count: events.length },
  ];

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-gray-700/50">
        <h3 className="text-sm font-medium text-gray-300">Collaboration</h3>
        <div className="flex items-center gap-1">
          {channelActiveDelegations.length > 0 && (
            <span className="px-1.5 py-0.5 text-xs bg-blue-500/20 text-blue-400 rounded-full">
              {channelActiveDelegations.length} active
            </span>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-700/50">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex-1 px-3 py-1.5 text-xs font-medium transition-colors ${
              activeTab === tab.key
                ? "text-blue-400 border-b-2 border-blue-400"
                : "text-gray-500 hover:text-gray-300"
            }`}
          >
            {tab.label}
            {tab.count > 0 && (
              <span className="ml-1 px-1 py-0.5 text-[10px] bg-gray-700 rounded-full">
                {tab.count}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {activeTab === "delegations" && (
          <DelegationsTab
            delegations={channelDelegations}
            loading={delegationsLoading}
            onCancel={async (id: string) => { await cancelDelegation(id); }}
            onRetry={async (id: string) => { await retry(id); }}
          />
        )}

        {activeTab === "artifacts" && (
          <ArtifactsTab
            artifacts={artifacts}
            loading={artifactsLoading}
          />
        )}

        {activeTab === "events" && (
          <EventsTab
            events={events}
            configs={configs}
            pushUrl={pushUrl}
            showPushConfig={showPushConfig}
            onPushUrlChange={setPushUrl}
            onTogglePushConfig={setShowPushConfig}
            onRegisterPushUrl={registerPushUrl}
            onClearEvents={clearEvents}
          />
        )}
      </div>

      {/* Push event toasts */}
      {events.length > 0 && (
        <PushEventToast event={events[0]} />
      )}
    </div>
  );
}

// ===========================================================================
// Delegations Tab
// ===========================================================================

interface DelegationsTabProps {
  delegations: Array<{
    id: string;
    from_agent_id: string;
    to_agent_id: string;
    task_description: string;
    status: string;
    result: string | null;
    error: string | null;
    created_at: string;
  }>;
  loading: boolean;
  onCancel: (id: string) => Promise<void>;
  onRetry: (id: string) => Promise<void>;
}

function DelegationsTab({ delegations, loading, onCancel, onRetry }: DelegationsTabProps) {
  if (loading) {
    return <div className="text-xs text-gray-500 text-center py-4">Loading...</div>;
  }

  if (delegations.length === 0) {
    return (
      <div className="text-xs text-gray-500 text-center py-4">
        No delegations yet. Use @agent mentions to delegate tasks.
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {delegations.map((d) => (
        <AgentTaskCard
          key={d.id}
          delegation={d}
          onCancel={() => onCancel(d.id)}
          onRetry={() => onRetry(d.id)}
        />
      ))}
    </div>
  );
}

// ===========================================================================
// Artifacts Tab
// ===========================================================================

interface ArtifactsTabProps {
  artifacts: Array<{
    id: string;
    name: string;
    producer_agent_id: string;
    mime_type: string | null;
    size: number;
    created_at: string;
  }>;
  loading: boolean;
}

function ArtifactsTab({ artifacts, loading }: ArtifactsTabProps) {
  if (loading) {
    return <div className="text-xs text-gray-500 text-center py-4">Loading...</div>;
  }

  if (artifacts.length === 0) {
    return (
      <div className="text-xs text-gray-500 text-center py-4">
        No artifacts produced yet.
      </div>
    );
  }

  // Group by agent
  const grouped = new Map<string, typeof artifacts>();
  for (const a of artifacts) {
    const existing = grouped.get(a.producer_agent_id) || [];
    existing.push(a);
    grouped.set(a.producer_agent_id, existing);
  }

  return (
    <div className="space-y-3">
      {Array.from(grouped.entries()).map(([agentId, agentArtifacts]) => (
        <div key={agentId}>
          <h4 className="text-xs font-medium text-gray-400 mb-1">
            {agentId} ({agentArtifacts.length})
          </h4>
          <div className="space-y-1">
            {agentArtifacts.map((a) => (
              <div
                key={a.id}
                className="flex items-center gap-2 px-2 py-1.5 bg-gray-800/50 rounded text-xs"
              >
                <span className="text-gray-200 truncate flex-1">{a.name}</span>
                <span className="text-gray-500">
                  {a.size > 1024 ? `${(a.size / 1024).toFixed(1)}KB` : `${a.size}B`}
                </span>
                {a.mime_type && (
                  <span className="text-gray-600 text-[10px]">{a.mime_type.split("/")[1]}</span>
                )}
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

// ===========================================================================
// Events Tab
// ===========================================================================

interface EventsTabProps {
  events: Array<{
    event_type: string;
    agent_id: string;
    task_id: string;
    message: string;
    timestamp: string;
  }>;
  configs: Array<{ id: string; url: string; active: boolean }>;
  pushUrl: string;
  showPushConfig: boolean;
  onPushUrlChange: (url: string) => void;
  onTogglePushConfig: (show: boolean) => void;
  onRegisterPushUrl: (url: string, token?: string, hmacSecret?: string) => Promise<unknown>;
  onClearEvents: () => void;
}

function EventsTab({
  events,
  configs,
  pushUrl,
  showPushConfig,
  onPushUrlChange,
  onTogglePushConfig,
  onRegisterPushUrl,
  onClearEvents,
}: EventsTabProps) {
  return (
    <div className="space-y-3">
      {/* Push config */}
      <div>
        <button
          onClick={() => onTogglePushConfig(!showPushConfig)}
          className="text-xs text-blue-400 hover:text-blue-300"
        >
          {showPushConfig ? "Hide" : "Show"} Push Config ({configs.length})
        </button>
        {showPushConfig && (
          <div className="mt-2 space-y-2">
            <div className="flex gap-1">
              <input
                type="text"
                value={pushUrl}
                onChange={(e) => onPushUrlChange(e.target.value)}
                placeholder="http://localhost:9470/push"
                className="flex-1 px-2 py-1 bg-gray-800 border border-gray-600 rounded text-xs text-gray-200"
              />
              <button
                onClick={() => onRegisterPushUrl(pushUrl)}
                className="px-2 py-1 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded"
              >
                Add
              </button>
            </div>
            {configs.map((c) => (
              <div key={c.id} className="flex items-center gap-2 text-xs text-gray-400">
                <span className={c.active ? "text-green-400" : "text-gray-600"}>
                  {c.active ? "●" : "○"}
                </span>
                <span className="truncate">{c.url}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Events list */}
      <div className="flex items-center justify-between">
        <h4 className="text-xs font-medium text-gray-400">Recent Events</h4>
        {events.length > 0 && (
          <button
            onClick={onClearEvents}
            className="text-[10px] text-gray-500 hover:text-gray-300"
          >
            Clear
          </button>
        )}
      </div>

      {events.length === 0 ? (
        <div className="text-xs text-gray-500 text-center py-2">
          No push events received yet.
        </div>
      ) : (
        <div className="space-y-1">
          {events.map((e, i) => (
            <div
              key={i}
              className="px-2 py-1.5 bg-gray-800/50 rounded text-xs"
            >
              <div className="flex items-center gap-2">
                <span
                  className={`text-[10px] font-mono ${
                    e.event_type === "task_completed"
                      ? "text-green-400"
                      : e.event_type === "task_failed"
                        ? "text-red-400"
                        : "text-blue-400"
                  }`}
                >
                  {e.event_type}
                </span>
                <span className="text-gray-400">{e.agent_id}</span>
              </div>
              {e.message && <p className="text-gray-500 mt-0.5">{e.message}</p>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default CollaborationView;
