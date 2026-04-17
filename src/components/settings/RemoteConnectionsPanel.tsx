/**
 * RemoteConnectionsPanel -- UI for managing remote A2A endpoint connections.
 *
 * Provides a card-based list of connections with add/edit/delete/test actions.
 * Uses brutalist design system (brutal-border, brutal-shadow, brutal-btn).
 */

import React, { useState } from "react";
import { useRemoteConnections } from "../../lib/useRemoteConnections";
import BridgeWorkspacePanel from "./BridgeWorkspacePanel";
import type {
  RemoteConnectionInfo,
  RemoteConnectionStatus,
  CreateRemoteConnectionRequest,
} from "../../types";
import { Plus, Pencil, Trash2, CheckCircle, AlertCircle } from "lucide-react";

// ===========================================================================
// Status badge component
// ===========================================================================

function StatusBadge({ status }: { status: RemoteConnectionStatus }) {
  const config: Record<RemoteConnectionStatus, { color: string; label: string }> = {
    online: { color: "bg-brutal-green", label: "Online" },
    offline: { color: "bg-gray-400", label: "Offline" },
    error: { color: "bg-brutal-pink", label: "Error" },
    unknown: { color: "bg-brutal-yellow", label: "Unknown" },
  };

  const { color, label } = config[status];

  return (
    <span className="inline-flex items-center gap-1.5 text-xs font-bold">
      <span className={`w-2 h-2 rounded-full ${color}`} />
      {label}
    </span>
  );
}

// ===========================================================================
// Connection form dialog
// ===========================================================================

interface ConnectionFormData {
  name: string;
  endpoint_url: string;
  auth_type: "none" | "api_key";
  api_key: string;
}

function ConnectionForm({
  initial,
  onSubmit,
  onCancel,
  submitLabel,
}: {
  initial?: ConnectionFormData;
  onSubmit: (data: ConnectionFormData) => void;
  onCancel: () => void;
  submitLabel: string;
}) {
  const [form, setForm] = useState<ConnectionFormData>(
    initial ?? {
      name: "",
      endpoint_url: "",
      auth_type: "none",
      api_key: "",
    }
  );

  const isValid =
    form.name.trim().length > 0 &&
    form.endpoint_url.trim().length > 0 &&
    (form.auth_type !== "api_key" || form.api_key.trim().length > 0);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (isValid) {
      onSubmit(form);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <div>
        <label className="block text-xs font-bold mb-1">
          Name
        </label>
        <input
          type="text"
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          placeholder="My Dev Server"
          className="w-full brutal-border bg-white px-3 py-1.5 text-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-black"
        />
      </div>

      <div>
        <label className="block text-xs font-bold mb-1">
          Endpoint URL
        </label>
        <input
          type="text"
          value={form.endpoint_url}
          onChange={(e) => setForm({ ...form, endpoint_url: e.target.value })}
          placeholder="https://dev-server:8443/a2a"
          className="w-full brutal-border bg-white px-3 py-1.5 text-sm font-mono placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-black"
        />
      </div>

      <div>
        <label className="block text-xs font-bold mb-1">
          Authentication
        </label>
        <div className="flex gap-3">
          <label className="inline-flex items-center gap-1.5 text-sm font-bold cursor-pointer">
            <input
              type="radio"
              name="auth_type"
              value="none"
              checked={form.auth_type === "none"}
              onChange={() => setForm({ ...form, auth_type: "none" })}
              className="accent-black"
            />
            None
          </label>
          <label className="inline-flex items-center gap-1.5 text-sm font-bold cursor-pointer">
            <input
              type="radio"
              name="auth_type"
              value="api_key"
              checked={form.auth_type === "api_key"}
              onChange={() => setForm({ ...form, auth_type: "api_key" })}
              className="accent-black"
            />
            API Key
          </label>
        </div>
      </div>

      {form.auth_type === "api_key" && (
        <div>
          <label className="block text-xs font-bold mb-1">
            API Key
          </label>
          <input
            type="password"
            value={form.api_key}
            onChange={(e) => setForm({ ...form, api_key: e.target.value })}
            placeholder="sk-..."
            className="w-full brutal-border bg-white px-3 py-1.5 text-sm font-mono placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-black"
          />
        </div>
      )}

      <div className="flex gap-2 pt-2">
        <button
          type="submit"
          disabled={!isValid}
          className="brutal-btn bg-brutal-cyan text-black text-xs disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {submitLabel}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="brutal-btn bg-white text-black text-xs"
        >
          Cancel
        </button>
      </div>
    </form>
  );
}

// ===========================================================================
// Connection card
// ===========================================================================

function ConnectionCard({
  conn,
  onTest,
  onEdit,
  onDelete,
  testing,
  testResult,
}: {
  conn: RemoteConnectionInfo;
  onTest: () => void;
  onEdit: () => void;
  onDelete: () => void;
  testing: boolean;
  testResult: { success: boolean; error?: string | null } | null;
}) {
  return (
    <div className="brutal-card">
      <div className="flex items-start justify-between mb-2">
        <div>
          <h4 className="text-sm font-extrabold">{conn.name}</h4>
          <p className="text-xs text-gray-500 font-mono mt-0.5">
            {conn.endpoint_url}
          </p>
        </div>
        <StatusBadge status={conn.status} />
      </div>

      {conn.agent_card && (
        <div className="mt-2 text-xs text-gray-600">
          <span className="font-bold">Agent:</span>{" "}
          {conn.agent_card.name}
          {conn.agent_card.version && (
            <span className="text-gray-400 ml-2">v{conn.agent_card.version}</span>
          )}
        </div>
      )}

      {testResult && !testResult.success && testResult.error && (
        <div className="mt-2 text-xs font-bold text-brutal-pink bg-brutal-pink/10 brutal-border px-2 py-1 flex items-start gap-1">
          <AlertCircle size={12} className="shrink-0 mt-0.5" />
          {testResult.error}
        </div>
      )}

      {testResult && testResult.success && (
        <div className="mt-2 text-xs font-bold text-brutal-green bg-brutal-green/10 brutal-border px-2 py-1 flex items-center gap-1">
          <CheckCircle size={12} />
          Connection successful
        </div>
      )}

      <div className="flex gap-2 mt-3">
        <button
          onClick={onTest}
          disabled={testing}
          className="brutal-btn bg-brutal-yellow text-black text-xs disabled:opacity-40"
        >
          {testing ? "Testing..." : "Test"}
        </button>
        <button
          onClick={onEdit}
          className="brutal-btn bg-white text-black text-xs"
        >
          <Pencil size={10} className="inline mr-1" />
          Edit
        </button>
        <button
          onClick={onDelete}
          className="brutal-btn bg-brutal-pink text-white text-xs"
        >
          <Trash2 size={10} className="inline mr-1" />
          Delete
        </button>
      </div>

      {conn.last_health_check_at && (
        <div className="mt-2 text-[10px] text-gray-400">
          Last check: {new Date(conn.last_health_check_at).toLocaleString()}
        </div>
      )}

      {/* Bridge Workspace Panel — shows if remote supports bridge.* operations */}
      {conn.status === "online" && (
        <BridgeWorkspacePanel connection={conn} />
      )}
    </div>
  );
}

// ===========================================================================
// Main panel
// ===========================================================================

export default function RemoteConnectionsPanel() {
  const {
    connections,
    loading,
    error,
    create,
    update,
    remove,
    test,
    healthCheckAll,
    testResults,
  } = useRemoteConnections();

  const [showAddForm, setShowAddForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  const handleAdd = async (data: ConnectionFormData) => {
    const request: CreateRemoteConnectionRequest = {
      name: data.name,
      endpoint_url: data.endpoint_url,
      auth_type: data.auth_type,
      api_key: data.auth_type === "api_key" ? data.api_key : undefined,
    };
    await create(request);
    setShowAddForm(false);
  };

  const handleEdit = (conn: RemoteConnectionInfo) => {
    setEditingId(conn.id);
  };

  const handleEditSubmit = async (
    id: string,
    data: ConnectionFormData
  ) => {
    await update(id, {
      name: data.name,
      endpoint_url: data.endpoint_url,
      auth_type: data.auth_type,
      api_key: data.auth_type === "api_key" ? data.api_key : undefined,
    });
    setEditingId(null);
  };

  const handleTest = async (id: string) => {
    setTestingId(id);
    await test(id);
    setTestingId(null);
  };

  const handleDelete = async (id: string) => {
    await remove(id);
    setDeleteConfirmId(null);
  };

  if (loading) {
    return (
      <div className="p-4 text-gray-500 text-sm font-bold">
        Loading connections...
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-extrabold uppercase tracking-wider">
          Remote Connections
        </h3>
        <div className="flex gap-2">
          <button
            onClick={healthCheckAll}
            className="brutal-btn bg-white text-black text-xs"
          >
            Check All
          </button>
          <button
            onClick={() => setShowAddForm(true)}
            className="brutal-btn bg-brutal-cyan text-black text-xs flex items-center gap-1"
          >
            <Plus size={12} />
            Add
          </button>
        </div>
      </div>

      {error && (
        <div className="text-xs font-bold text-brutal-pink bg-brutal-pink/10 brutal-border px-3 py-2">
          {error}
        </div>
      )}

      {showAddForm && (
        <div className="brutal-card bg-brutal-bg">
          <h4 className="text-sm font-extrabold mb-3">
            Add Connection
          </h4>
          <ConnectionForm
            onSubmit={handleAdd}
            onCancel={() => setShowAddForm(false)}
            submitLabel="Save"
          />
        </div>
      )}

      {connections.length === 0 && !showAddForm && (
        <div className="brutal-card bg-brutal-bg text-center py-8">
          <p className="text-sm text-gray-500 font-bold">No remote connections configured.</p>
          <p className="text-xs text-gray-400 mt-1">
            Click "+ Add" to connect to a remote A2A endpoint.
          </p>
        </div>
      )}

      <div className="space-y-3">
        {connections.map((conn) => {
          if (editingId === conn.id) {
            return (
              <div
                key={conn.id}
                className="brutal-card bg-brutal-yellow/10"
              >
                <h4 className="text-sm font-extrabold mb-3">
                  Edit Connection
                </h4>
                <ConnectionForm
                  initial={{
                    name: conn.name,
                    endpoint_url: conn.endpoint_url,
                    auth_type: (conn.auth_type as "none" | "api_key") || "none",
                    api_key: "",
                  }}
                  onSubmit={(data) => handleEditSubmit(conn.id, data)}
                  onCancel={() => setEditingId(null)}
                  submitLabel="Update"
                />
              </div>
            );
          }

          if (deleteConfirmId === conn.id) {
            return (
              <div
                key={conn.id}
                className="brutal-card bg-brutal-pink/10"
              >
                <p className="text-sm font-extrabold mb-3">
                  Delete "{conn.name}"?
                </p>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleDelete(conn.id)}
                    className="brutal-btn bg-brutal-pink text-white text-xs"
                  >
                    Confirm Delete
                  </button>
                  <button
                    onClick={() => setDeleteConfirmId(null)}
                    className="brutal-btn bg-white text-black text-xs"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            );
          }

          return (
            <ConnectionCard
              key={conn.id}
              conn={conn}
              onTest={() => handleTest(conn.id)}
              onEdit={() => handleEdit(conn)}
              onDelete={() => setDeleteConfirmId(conn.id)}
              testing={testingId === conn.id}
              testResult={testResults.get(conn.id) ?? null}
            />
          );
        })}
      </div>
    </div>
  );
}
