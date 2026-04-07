import React from 'react';
import { X, Send, Image as ImageIcon, User } from 'lucide-react';
import { cn } from '../lib/utils';

interface ThreadPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ThreadPanel: React.FC<ThreadPanelProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  return (
    <div className="w-80 h-full bg-white brutal-border-l flex flex-col">
      {/* Header */}
      <div className="p-3 brutal-border-b flex items-center justify-between bg-gray-50">
        <div className="font-black text-sm truncate">
          Thread — <span className="text-gray-500">#kagent-integrate-sap-ai-core</span>
        </div>
        <button onClick={onClose} className="p-1 brutal-border hover:bg-gray-200">
          <X size={16} />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6 bg-brutal-bg/30">
        {/* Code Block Example */}
        <div className="brutal-border bg-black text-white p-3 font-mono text-[10px] leading-tight overflow-x-auto">
          <pre>{`metadata:
  name: production-cluster-eu
spec:
  apiServer: https://cluster-eu.e
  mcpEndpoint: http://cluster-eu-
  region: eu-central-1
  labels:
    env: production
    region: eu`}</pre>
        </div>

        {/* Table Example */}
        <div className="space-y-2">
          <h4 className="font-black text-xs italic">五、总结</h4>
          <div className="brutal-border overflow-hidden">
            <table className="w-full text-[10px] border-collapse">
              <thead>
                <tr className="bg-brutal-cyan brutal-border-b">
                  <th className="p-1 text-left brutal-border-r">方面</th>
                  <th className="p-1 text-left">kubectl-mcp-server + kubeconfig</th>
                </tr>
              </thead>
              <tbody>
                <tr className="brutal-border-b">
                  <td className="p-1 brutal-border-r bg-gray-100">架构类型</td>
                  <td className="p-1">单点多连接</td>
                </tr>
                <tr className="brutal-border-b">
                  <td className="p-1 brutal-border-r bg-gray-100">故障隔离</td>
                  <td className="p-1">❌ 单点故障</td>
                </tr>
                <tr className="brutal-border-b">
                  <td className="p-1 brutal-border-r bg-gray-100">扩展性</td>
                  <td className="p-1">❌ 受限</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        {/* Message Example */}
        <div className="flex gap-2">
          <div className="w-8 h-8 brutal-border bg-purple-400 flex items-center justify-center shrink-0">
            <User size={16} />
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-black text-xs">Lissa</span>
              <span className="text-[8px] text-gray-500 uppercase">owner 08:23 AM</span>
            </div>
            <div className="text-xs leading-relaxed">
              @克劳德 你来review一下 上面Alice的方案设计 看看有没有什么问题？
            </div>
          </div>
        </div>
      </div>

      {/* Input */}
      <div className="p-3 brutal-border-t bg-white">
        <div className="brutal-border p-2 min-h-[80px] text-xs text-gray-400 mb-2">
          Message thread
        </div>
        <div className="flex items-center justify-between">
          <button className="p-1.5 brutal-border hover:bg-gray-100">
            <ImageIcon size={16} />
          </button>
          <button className="brutal-btn bg-brutal-pink text-white text-[10px] flex items-center gap-1 opacity-50 cursor-not-allowed">
            Send <Send size={10} />
          </button>
        </div>
      </div>
    </div>
  );
};
