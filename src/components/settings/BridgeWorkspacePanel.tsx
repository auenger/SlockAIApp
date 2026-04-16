import { useState } from 'react';
import type { BridgeAgent, BridgeFileEntry, RemoteConnectionInfo } from '../../types';
import { useBridgeWorkspace } from '../../lib/useBridgeWorkspace';

interface Props {
  connection: RemoteConnectionInfo;
}

export default function BridgeWorkspacePanel({ connection }: Props) {
  const { isBridge, workspaceInfo, agents, loading, error, listFiles, readFile } =
    useBridgeWorkspace(connection);

  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);
  const [files, setFiles] = useState<BridgeFileEntry[]>([]);
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [filesLoading, setFilesLoading] = useState(false);

  if (!isBridge) return null;

  const handleAgentClick = async (agent: BridgeAgent) => {
    setSelectedAgent(agent.agent_id);
    setFileContent(null);
    setFilesLoading(true);
    try {
      const entries = await listFiles(agent.agent_id);
      setFiles(entries);
    } catch {
      setFiles([]);
    } finally {
      setFilesLoading(false);
    }
  };

  const handleFileClick = async (entry: BridgeFileEntry) => {
    if (entry.is_dir || !selectedAgent) return;
    try {
      const result = await readFile(selectedAgent, entry.name);
      setFileContent(result.content);
    } catch {
      setFileContent(null);
    }
  };

  return (
    <div className="mt-3 rounded-lg border border-white/10 bg-white/5 p-3">
      <div className="mb-2 flex items-center gap-2">
        <span className="text-xs font-medium text-zinc-400">Bridge Workspace</span>
        {loading && (
          <span className="text-xs text-zinc-500">Loading...</span>
        )}
      </div>

      {error && (
        <div className="mb-2 text-xs text-red-400">{error}</div>
      )}

      {workspaceInfo && (
        <div className="mb-2 text-xs text-zinc-500">
          {workspaceInfo.total_agents} agents ({workspaceInfo.workspace_root})
        </div>
      )}

      {/* Agent cards */}
      <div className="space-y-1">
        {agents.map((agent) => (
          <button
            key={agent.agent_id}
            onClick={() => handleAgentClick(agent)}
            className={`flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm transition-colors ${
              selectedAgent === agent.agent_id
                ? 'bg-white/10 text-white'
                : 'text-zinc-300 hover:bg-white/5'
            }`}
          >
            <span className="text-base">{agent.emoji}</span>
            <span className="flex-1 truncate">{agent.name}</span>
            <span className="text-xs text-zinc-500">{agent.runtime_type}</span>
          </button>
        ))}
      </div>

      {/* File browser */}
      {selectedAgent && (
        <div className="mt-3 border-t border-white/10 pt-2">
          <div className="mb-1 text-xs font-medium text-zinc-400">
            Files ({selectedAgent})
          </div>
          {filesLoading ? (
            <div className="text-xs text-zinc-500">Loading files...</div>
          ) : (
            <div className="max-h-48 overflow-y-auto space-y-0.5">
              {files.map((entry) => (
                <button
                  key={entry.name}
                  onClick={() => handleFileClick(entry)}
                  className="flex w-full items-center gap-1.5 rounded px-1.5 py-0.5 text-xs text-zinc-400 hover:bg-white/5 hover:text-zinc-300"
                >
                  <span>{entry.is_dir ? '📁' : '📄'}</span>
                  <span className="flex-1 truncate">{entry.name}</span>
                  {!entry.is_dir && (
                    <span className="text-zinc-600">
                      {formatSize(entry.size)}
                    </span>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* File content viewer */}
      {fileContent !== null && (
        <div className="mt-2 border-t border-white/10 pt-2">
          <div className="max-h-64 overflow-y-auto rounded bg-black/20 p-2">
            <pre className="whitespace-pre-wrap text-xs text-zinc-300">
              {fileContent}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}K`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}M`;
}
