import React, { useState, useEffect } from 'react';
import { Modal } from './Modals';
import { IconPicker } from './IconPicker';
import { createAgent } from '../lib/ipc';
import { useRuntimeStatus } from '../lib/useRuntimeStatus';
import type { CreateAgentRequest, RuntimeType, AgentRuntimeInfo } from '../types';

interface CreateAgentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

/**
 * Map runtime info to a human-readable display name.
 */
function getRuntimeDisplayName(rt: AgentRuntimeInfo): string {
  return rt.name || rt.runtime_type;
}

/**
 * Map runtime info to a short status badge label.
 */
function getRuntimeBadge(rt: AgentRuntimeInfo): { label: string; color: string } {
  switch (rt.status) {
    case 'available':
      return { label: rt.version ? `v${rt.version}` : 'Available', color: 'text-brutal-green' };
    case 'unhealthy':
      return { label: 'Unhealthy', color: 'text-yellow-500' };
    case 'not-installed':
      return { label: 'Not Installed', color: 'text-gray-400' };
    case 'detecting':
      return { label: 'Detecting...', color: 'text-brutal-cyan' };
    default:
      return { label: 'Unknown', color: 'text-gray-400' };
  }
}

/**
 * Modal for creating a new Agent.
 * Follows brutal-border new brutalist style.
 * Includes runtime type selection with availability status.
 */
export const CreateAgentModal: React.FC<CreateAgentModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
}) => {
  const [name, setName] = useState('');
  const [icon, setIcon] = useState<string | null>(null);
  const [creature, setCreature] = useState('AI');
  const [vibe, setVibe] = useState('helpful');
  const [runtimeType, setRuntimeType] = useState<RuntimeType>('claude_code');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Runtime status detection
  const { runtimes, scanning, refresh: scanRuntimes } = useRuntimeStatus();

  // Auto-scan runtimes when modal opens
  useEffect(() => {
    if (isOpen) {
      scanRuntimes();
    }
  }, [isOpen, scanRuntimes]);

  const handleSubmit = async () => {
    if (!name.trim()) return;

    setLoading(true);
    setError(null);

    try {
      const request: CreateAgentRequest = {
        name: name.trim(),
        icon: icon || undefined,
        emoji: icon ? undefined : 'robot',
        creature: creature.trim() || 'AI',
        vibe: vibe.trim() || 'helpful',
        runtime_type: runtimeType,
      };

      await createAgent(request);
      onSuccess?.();
      handleClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setName('');
    setIcon(null);
    setCreature('AI');
    setVibe('helpful');
    setRuntimeType('claude_code');
    setError(null);
    setLoading(false);
    onClose();
  };

  const isDisabled = !name.trim() || loading;

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title="Create Agent"
      footer={
        <>
          <button
            onClick={handleClose}
            className="brutal-btn bg-white"
            disabled={loading}
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isDisabled}
            className={`brutal-btn ${isDisabled ? 'bg-gray-200 text-gray-400 cursor-not-allowed' : 'bg-brutal-pink text-white'}`}
          >
            {loading ? 'Creating...' : 'Create'}
          </button>
        </>
      }
    >
      <div className="space-y-4">
        {/* Name - Required */}
        <div>
          <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
            Name <span className="text-brutal-pink">*</span>
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Agent name"
            className="w-full brutal-border p-3 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
            autoFocus
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !isDisabled) handleSubmit();
            }}
          />
        </div>

        {/* Icon */}
        <div>
          <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
            Icon
          </label>
          <IconPicker
            value={icon}
            onChange={setIcon}
          />
        </div>

        {/* Runtime Selection */}
        <div>
          <label className="block text-[10px] font-black uppercase text-gray-500 mb-2">
            Runtime
          </label>
          {scanning && runtimes.length === 0 ? (
            <div className="flex items-center gap-2 p-3 brutal-border bg-gray-50">
              <div className="w-4 h-4 border-2 border-brutal-cyan border-t-transparent rounded-full animate-spin" />
              <span className="text-xs font-bold text-gray-400">Detecting runtimes...</span>
            </div>
          ) : (
            <div className="space-y-1.5">
              {runtimes.map((rt) => {
                const isSelected = runtimeType === rt.runtime_type;
                const badge = getRuntimeBadge(rt);
                const isAvailable = rt.status === 'available';

                return (
                  <label
                    key={rt.id}
                    className={`
                      flex items-center gap-3 p-2.5 brutal-border cursor-pointer transition-colors
                      ${isSelected ? 'bg-brutal-cyan/10 border-black' : 'bg-white hover:bg-gray-50'}
                      ${!isAvailable ? 'opacity-70' : ''}
                    `}
                  >
                    <input
                      type="radio"
                      name="runtime"
                      value={rt.runtime_type}
                      checked={isSelected}
                      onChange={() => setRuntimeType(rt.runtime_type)}
                      className="sr-only"
                    />
                    {/* Custom radio indicator */}
                    <div className={`
                      w-4 h-4 brutal-border rounded-full flex items-center justify-center shrink-0
                      ${isSelected ? 'bg-brutal-cyan' : 'bg-white'}
                    `}>
                      {isSelected && (
                        <div className="w-2 h-2 bg-black rounded-full" />
                      )}
                    </div>

                    {/* Runtime info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-black">{getRuntimeDisplayName(rt)}</span>
                        {isAvailable ? (
                          <span className={`text-[10px] font-bold ${badge.color}`}>
                            {badge.label}
                          </span>
                        ) : (
                          <span className={`text-[10px] font-bold ${badge.color}`}>
                            {badge.label}
                          </span>
                        )}
                      </div>
                      {!isAvailable && rt.install_hint && (
                        <div className="mt-0.5 text-[10px] font-mono text-gray-400 truncate" title={rt.install_hint}>
                          {rt.install_hint}
                        </div>
                      )}
                    </div>

                    {/* Status icon */}
                    <div className="shrink-0">
                      {isAvailable ? (
                        <span className="text-brutal-green text-sm font-bold">+</span>
                      ) : (
                        <span className="text-gray-300 text-sm font-bold">-</span>
                      )}
                    </div>
                  </label>
                );
              })}
            </div>
          )}
        </div>

        {/* Creature */}
        <div>
          <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
            Creature
          </label>
          <input
            type="text"
            value={creature}
            onChange={(e) => setCreature(e.target.value)}
            placeholder="AI"
            className="w-full brutal-border p-3 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
          />
        </div>

        {/* Vibe */}
        <div>
          <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
            Vibe
          </label>
          <input
            type="text"
            value={vibe}
            onChange={(e) => setVibe(e.target.value)}
            placeholder="helpful"
            className="w-full brutal-border p-3 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
          />
        </div>

        {/* Error message */}
        {error && (
          <div className="text-brutal-pink text-xs font-bold p-2 brutal-border bg-brutal-pink/10">
            {error}
          </div>
        )}
      </div>
    </Modal>
  );
};
