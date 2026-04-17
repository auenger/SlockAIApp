import { useState } from 'react';
import { ChevronDown, ChevronRight, Folder, FileText, Loader2 } from 'lucide-react';
import type { BridgeAgent, BridgeFileEntry, RemoteConnectionInfo } from '../../types';
import { useBridgeWorkspace } from '../../lib/useBridgeWorkspace';
import { AgentIcon } from '../AgentIcon';

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
  const [collapsed, setCollapsed] = useState(false);

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
    <div className="mt-3 brutal-border-t pt-3">
      {/* Header with collapse toggle */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="w-full flex items-center gap-1 text-xs font-extrabold uppercase tracking-wider hover:bg-gray-100 px-1 py-0.5 transition-colors"
      >
        {collapsed ? <ChevronRight size={12} /> : <ChevronDown size={12} />}
        Bridge Workspace
        {workspaceInfo && (
          <span className="text-gray-400 font-normal ml-1 normal-case">
            ({workspaceInfo.total_agents} agents)
          </span>
        )}
        {loading && <Loader2 size={10} className="animate-spin ml-1" />}
      </button>

      {collapsed ? null : (
        <div className="mt-2 max-h-80 overflow-y-auto">
          {error && (
            <div className="mb-2 text-xs font-bold text-brutal-pink">{error}</div>
          )}

          {/* Agent cards */}
          <div className="space-y-1">
            {agents.map((agent) => (
              <button
                key={agent.agent_id}
                onClick={() => handleAgentClick(agent)}
                className={`flex w-full items-center gap-2 brutal-border px-2 py-1.5 text-left text-sm transition-all ${
                  selectedAgent === agent.agent_id
                    ? 'bg-brutal-cyan/20 translate-x-[-1px] translate-y-[-1px] brutal-shadow-sm'
                    : 'border-transparent hover:border-black hover:bg-gray-50'
                }`}
              >
                <AgentIcon
                  emoji={agent.emoji}
                  size="sm"
                  bgColor="bg-brutal-cyan"
                />
                <span className="flex-1 truncate font-bold text-xs">{agent.name}</span>
                <span className="text-[10px] text-gray-400 font-mono">{agent.runtime_type}</span>
              </button>
            ))}
          </div>

          {/* File browser */}
          {selectedAgent && (
            <div className="mt-2 brutal-border-t pt-2">
              <div className="mb-1 text-xs font-extrabold uppercase tracking-wider">
                Files ({selectedAgent})
              </div>
              {filesLoading ? (
                <div className="flex items-center gap-1 text-xs text-gray-400">
                  <Loader2 size={10} className="animate-spin" />
                  Loading files...
                </div>
              ) : (
                <div className="max-h-40 overflow-y-auto space-y-0.5">
                  {files.map((entry) => (
                    <button
                      key={entry.name}
                      onClick={() => handleFileClick(entry)}
                      className="flex w-full items-center gap-1.5 px-1.5 py-0.5 text-xs hover:bg-gray-100 transition-colors"
                    >
                      {entry.is_dir ? (
                        <Folder size={12} className="text-brutal-yellow shrink-0" />
                      ) : (
                        <FileText size={12} className="text-gray-400 shrink-0" />
                      )}
                      <span className={`flex-1 truncate ${entry.is_dir ? 'font-bold' : 'text-gray-600'}`}>{entry.name}</span>
                      {!entry.is_dir && (
                        <span className="text-gray-400 font-mono text-[10px]">
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
            <div className="mt-2 brutal-border-t pt-2">
              <div className="max-h-48 overflow-y-auto brutal-border bg-brutal-bg p-2">
                <pre className="whitespace-pre-wrap text-xs font-mono">
                  {fileContent}
                </pre>
              </div>
            </div>
          )}
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
