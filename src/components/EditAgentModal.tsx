import React, { useState, useEffect } from 'react';
import { Modal } from './Modals';
import { IconPicker } from './IconPicker';
import { updateAgent, getAgentIdentity } from '../lib/ipc';
import type { UpdateAgentRequest, IdentitySummary } from '../types';

interface EditAgentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  /** The agent ID to edit */
  agentId: string | null;
}

/**
 * Modal for editing an existing Agent's properties.
 *
 * Pre-fills the current agent's identity data and allows the user
 * to modify name, icon, creature, vibe. Saving calls updateAgent IPC.
 */
export const EditAgentModal: React.FC<EditAgentModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
  agentId,
}) => {
  const [name, setName] = useState('');
  const [icon, setIcon] = useState<string | null>(null);
  const [creature, setCreature] = useState('');
  const [vibe, setVibe] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadingIdentity, setLoadingIdentity] = useState(false);

  // Original values for tracking changes
  const [original, setOriginal] = useState<IdentitySummary | null>(null);

  // Load identity when modal opens
  useEffect(() => {
    if (isOpen && agentId) {
      setLoadingIdentity(true);
      setError(null);
      getAgentIdentity(agentId)
        .then((identity) => {
          setName(identity.name);
          setIcon(identity.icon);
          setCreature(identity.creature);
          setVibe(identity.vibe);
          setOriginal(identity);
        })
        .catch((err) => {
          setError(err instanceof Error ? err.message : String(err));
        })
        .finally(() => {
          setLoadingIdentity(false);
        });
    }
  }, [isOpen, agentId]);

  const handleSubmit = async () => {
    if (!name.trim() || !agentId) return;

    setLoading(true);
    setError(null);

    try {
      // Build the update request with only changed fields
      const request: UpdateAgentRequest = {};

      if (original && name.trim() !== original.name) {
        request.name = name.trim();
      }
      if (original && creature.trim() !== original.creature) {
        request.creature = creature.trim();
      }
      if (original && vibe.trim() !== original.vibe) {
        request.vibe = vibe.trim();
      }
      // icon needs special handling for null vs string
      const currentIcon = icon || '';
      const originalIcon = original?.icon || '';
      if (currentIcon !== originalIcon) {
        request.icon = currentIcon;
      }

      // Always send the request (backend handles partial updates)
      await updateAgent(agentId, request);
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
    setCreature('');
    setVibe('');
    setError(null);
    setLoading(false);
    setLoadingIdentity(false);
    setOriginal(null);
    onClose();
  };

  const isDisabled = !name.trim() || loading || loadingIdentity;

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title="Edit Agent"
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
            {loading ? 'Saving...' : 'Save'}
          </button>
        </>
      }
    >
      <div className="space-y-4">
        {loadingIdentity ? (
          <div className="flex items-center justify-center py-8">
            <div className="w-6 h-6 border-2 border-brutal-cyan border-t-transparent rounded-full animate-spin" />
            <span className="ml-2 text-xs font-bold text-gray-400">Loading agent data...</span>
          </div>
        ) : (
          <>
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
          </>
        )}

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
