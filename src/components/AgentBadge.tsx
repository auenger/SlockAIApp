import React from 'react';
import { Cloud, CloudOff } from 'lucide-react';
import { cn } from '../lib/utils';
import { AgentIcon } from './AgentIcon';
import type { AgentWithRuntime } from '../types';
import { isRemoteAgent, getConnectionId } from '../lib/useAllAgents';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface AgentBadgeProps {
  /** Agent with runtime info */
  agent: AgentWithRuntime;
  /** Connection ID → connection name mapping (for showing source) */
  connectionNames?: Map<string, string>;
  /** Show remote connection badge */
  showConnectionBadge?: boolean;
  /** Show online status indicator */
  showStatus?: boolean;
  /** Size preset for the icon */
  size?: 'sm' | 'md' | 'lg';
  /** Background color for the icon */
  bgColor?: string;
  /** Additional CSS classes */
  className?: string;
  /** Click handler */
  onClick?: () => void;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const AgentBadge: React.FC<AgentBadgeProps> = ({
  agent,
  connectionNames,
  showConnectionBadge = true,
  showStatus = true,
  size = 'md',
  bgColor,
  className,
  onClick,
}) => {
  const remote = isRemoteAgent(agent.agent);
  const connId = remote ? getConnectionId(agent.agent) : null;
  const connName = connId ? connectionNames?.get(connId) : null;

  // Remote agents use a distinct color when not overridden
  const iconBgColor = bgColor ?? (remote ? 'bg-purple-300' : 'bg-brutal-cyan');

  // Offline remote agents get muted styling
  const isOffline = remote && agent.runtime_status === 'not-installed';

  return (
    <div
      className={cn(
        "flex items-center gap-2",
        isOffline && "opacity-50",
        onClick && "cursor-pointer",
        className
      )}
      onClick={onClick}
    >
      <div className="relative">
        <AgentIcon
          icon={agent.agent.icon}
          emoji={agent.agent.emoji}
          size={size}
          bgColor={iconBgColor}
        />
        {/* Remote badge — cloud icon in bottom-right corner */}
        {showConnectionBadge && remote && (
          <div
            className={cn(
              "absolute -bottom-0.5 -right-0.5 brutal-border bg-white p-0",
              isOffline ? "text-gray-400" : "text-purple-500"
            )}
            title={connName ? `Remote: ${connName}` : 'Remote Agent'}
          >
            {isOffline ? <CloudOff size={8} /> : <Cloud size={8} />}
          </div>
        )}
      </div>
      <div className="flex-1 min-w-0">
        <div className={cn("font-black truncate", size === 'sm' ? 'text-[10px]' : 'text-sm')}>
          {agent.agent.name}
        </div>
        {showConnectionBadge && remote && connName && (
          <div className="text-[8px] text-purple-400 font-mono truncate">
            via {connName}
          </div>
        )}
      </div>
      {showStatus && !remote && (
        <div
          className={cn(
            "w-2 h-2 rounded-full shrink-0 brutal-border",
            agent.runtime_status === 'available' ? 'bg-brutal-green' :
            agent.runtime_status === 'unhealthy' ? 'bg-brutal-yellow' :
            'bg-gray-400'
          )}
          title={agent.runtime_status}
        />
      )}
    </div>
  );
};
