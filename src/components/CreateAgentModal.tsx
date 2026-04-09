import React, { useState } from 'react';
import { Modal } from './Modals';
import { createAgent } from '../lib/ipc';
import type { CreateAgentRequest } from '../types';

interface CreateAgentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

/**
 * Modal for creating a new Agent.
 * Follows brutal-border new brutalist style.
 */
export const CreateAgentModal: React.FC<CreateAgentModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
}) => {
  const [name, setName] = useState('');
  const [emoji, setEmoji] = useState('robot');
  const [creature, setCreature] = useState('AI');
  const [vibe, setVibe] = useState('helpful');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    if (!name.trim()) return;

    setLoading(true);
    setError(null);

    try {
      const request: CreateAgentRequest = {
        name: name.trim(),
        emoji: emoji.trim() || 'robot',
        creature: creature.trim() || 'AI',
        vibe: vibe.trim() || 'helpful',
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
    setEmoji('robot');
    setCreature('AI');
    setVibe('helpful');
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

        {/* Emoji */}
        <div>
          <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
            Emoji
          </label>
          <input
            type="text"
            value={emoji}
            onChange={(e) => setEmoji(e.target.value)}
            placeholder="robot"
            className="w-full brutal-border p-3 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
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
