import React, { useState, useRef, useEffect } from 'react';
import { X, Send, User } from 'lucide-react';
import { AgentIcon } from './AgentIcon';
import type { Thread, ThreadMessageData, AgentWithRuntime } from '../types';

interface ThreadPanelProps {
  isOpen: boolean;
  thread: Thread | null;
  agent: AgentWithRuntime | null;
  onSend: (message: string) => void;
  onClose: () => void;
  /** Resizable width style from parent */
  style?: React.CSSProperties;
  /** Resize handle ref from parent */
  resizeHandleRef?: React.RefObject<HTMLDivElement | null>;
}

export const ThreadPanel: React.FC<ThreadPanelProps> = ({
  isOpen,
  thread,
  agent,
  onSend,
  onClose,
  style,
  resizeHandleRef,
}) => {
  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [thread?.messages]);

  const handleSend = () => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    onSend(trimmed);
    setInputValue('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // Format timestamp for display
  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString([], {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (!isOpen) return null;

  // Empty state: no thread selected
  if (!thread) {
    return (
      <div className="h-full bg-white brutal-border-l flex flex-col relative" style={style}>
        {/* Resize Handle — left edge */}
        <div
          ref={resizeHandleRef}
          className="absolute top-0 left-0 bottom-0 w-1 cursor-col-resize hover:bg-black/20 active:bg-black/30 transition-colors z-10"
        />
        <div className="p-3 brutal-border-b flex items-center justify-between bg-gray-50">
          <div className="font-black text-sm truncate">Thread</div>
          <button onClick={onClose} className="p-1 brutal-border hover:bg-gray-200">
            <X size={16} />
          </button>
        </div>
        <div className="flex-1 flex flex-col justify-center items-center text-gray-400 italic text-sm p-4 text-center">
          <MessageSquareIcon className="w-12 h-12 mb-3 opacity-30" />
          <p>Select a thread to view details</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full bg-white brutal-border-l flex flex-col relative" style={style}>
      {/* Resize Handle — left edge */}
      <div
        ref={resizeHandleRef}
        className="absolute top-0 left-0 bottom-0 w-1 cursor-col-resize hover:bg-black/20 active:bg-black/30 transition-colors z-10"
      />
      {/* Header */}
      <div className="p-3 brutal-border-b flex items-center justify-between bg-gray-50">
        <div className="font-black text-sm truncate flex-1 mr-2">
          <span className="text-gray-500">Thread — </span>
          <span className="truncate">{thread.title || 'Untitled'}</span>
        </div>
        <button onClick={onClose} className="p-1 brutal-border hover:bg-gray-200 shrink-0">
          <X size={16} />
        </button>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 bg-brutal-bg/30">
        {thread.messages.length === 0 ? (
          <div className="h-full flex flex-col justify-center items-center text-gray-400 italic text-sm">
            <p>No messages yet.</p>
            <p className="text-[10px] mt-1">Start the conversation below.</p>
          </div>
        ) : (
          thread.messages.map((msg: ThreadMessageData) => (
            <div key={msg.id} className="flex gap-2">
              {msg.role === 'user' ? (
                <div className="w-8 h-8 brutal-border bg-purple-400 flex items-center justify-center shrink-0">
                  <User size={16} className="text-white" />
                </div>
              ) : (
                <AgentIcon
                  icon={agent?.agent.icon}
                  emoji={agent?.agent.emoji}
                  size="md"
                  bgColor="bg-brutal-cyan"
                />
              )}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-black text-xs">
                    {msg.role === 'user' ? 'You' : (agent?.agent.name || 'Agent')}
                  </span>
                  <span className="text-[8px] text-gray-500 uppercase">
                    {formatTime(msg.timestamp)}
                  </span>
                </div>
                <div className="text-xs leading-relaxed whitespace-pre-wrap">
                  {msg.content}
                </div>
              </div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="p-3 brutal-border-t bg-white">
        <div className="brutal-border p-2 min-h-[80px]">
          <textarea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Message thread..."
            className="w-full h-full min-h-[60px] resize-none text-xs outline-none placeholder:text-gray-400"
          />
        </div>
        <div className="flex items-center justify-end mt-2">
          <button
            onClick={handleSend}
            disabled={!inputValue.trim()}
            className={`brutal-btn bg-brutal-pink text-white text-[10px] flex items-center gap-1 ${
              !inputValue.trim() ? 'opacity-50 cursor-not-allowed' : ''
            }`}
          >
            Send <Send size={10} />
          </button>
        </div>
      </div>
    </div>
  );
};

// Simple icon component for empty state
const MessageSquareIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={className}
  >
    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
  </svg>
);
