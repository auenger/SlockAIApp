/**
 * TaskAssignDropdown — Agent assignment selector for Tasks.
 *
 * Shows a dropdown of available agents with their status indicators.
 */

import React, { useState, useRef, useEffect } from 'react';
import { ChevronDown, X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { AgentIcon } from '../AgentIcon';
import type { AgentWithRuntime } from '../../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface TaskAssignDropdownProps {
  /** Available agents */
  agents: AgentWithRuntime[];
  /** Currently selected agent ID */
  value?: string | null;
  /** Change callback */
  onChange: (agentId: string | null) => void;
  /** Placeholder text */
  placeholder?: string;
  /** Additional CSS classes */
  className?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TaskAssignDropdown: React.FC<TaskAssignDropdownProps> = ({
  agents,
  value,
  onChange,
  placeholder = 'Assign to agent...',
  className,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const selectedAgent = agents.find(a => a.agent.agent_id === value);

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  return (
    <div ref={containerRef} className={cn('relative', className)}>
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center gap-2 brutal-border bg-white px-2 py-1.5 text-xs hover:bg-gray-50 transition-colors"
      >
        {selectedAgent ? (
          <>
            <AgentIcon
              icon={selectedAgent.agent.icon}
              emoji={selectedAgent.agent.emoji}
              size="sm"
              bgColor="bg-brutal-cyan"
            />
            <span className="font-bold flex-1 text-left truncate">{selectedAgent.agent.name}</span>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onChange(null);
              }}
              className="p-0.5 hover:bg-gray-200"
            >
              <X size={10} />
            </button>
          </>
        ) : (
          <span className="flex-1 text-left text-gray-400">{placeholder}</span>
        )}
        <ChevronDown size={12} className="text-gray-400 shrink-0" />
      </button>

      {isOpen && (
        <div className="absolute z-50 top-full left-0 right-0 mt-1 bg-white brutal-border brutal-shadow-sm max-h-48 overflow-y-auto">
          {agents.length === 0 ? (
            <div className="px-3 py-2 text-[10px] text-gray-400 italic">No agents available</div>
          ) : (
            agents.map((awr) => {
              const isSelected = awr.agent.agent_id === value;
              return (
                <button
                  key={awr.agent.agent_id}
                  type="button"
                  onClick={() => {
                    onChange(awr.agent.agent_id);
                    setIsOpen(false);
                  }}
                  className={cn(
                    'w-full flex items-center gap-2 px-2 py-1.5 text-xs hover:bg-gray-50 transition-colors',
                    isSelected && 'bg-brutal-yellow/30'
                  )}
                >
                  <AgentIcon
                    icon={awr.agent.icon}
                    emoji={awr.agent.emoji}
                    size="sm"
                    bgColor="bg-brutal-cyan"
                  />
                  <span className="font-bold flex-1 text-left truncate">{awr.agent.name}</span>
                  <span className="text-[9px] text-gray-400 font-mono">{awr.agent.agent_id}</span>
                </button>
              );
            })
          )}
        </div>
      )}
    </div>
  );
};
