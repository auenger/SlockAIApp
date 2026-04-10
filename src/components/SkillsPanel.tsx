/**
 * SkillFormModal - Add/Edit Skill dialog.
 *
 * A brutalist-style modal for creating or editing a Skill configuration.
 */

import React, { useState, useEffect } from 'react';
import { X, Zap } from 'lucide-react';
import { cn } from '../lib/utils';
import type { SkillInfo, SkillType } from '../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface SkillFormModalProps {
  /** Whether the modal is visible */
  open: boolean;
  /** Close handler */
  onClose: () => void;
  /** Submit handler (receives form data) */
  onSubmit: (data: { name: string; skill_type: SkillType; config: Record<string, unknown> }) => void;
  /** Optional existing skill to edit (null = create mode) */
  skill?: SkillInfo | null;
  /** Whether a submission is in progress */
  submitting?: boolean;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const SkillFormModal: React.FC<SkillFormModalProps> = ({
  open,
  onClose,
  onSubmit,
  skill,
  submitting = false,
}) => {
  const [name, setName] = useState('');
  const [skillType, setSkillType] = useState<SkillType>('Tool');
  const [configText, setConfigText] = useState('{}');

  // Pre-fill when editing
  useEffect(() => {
    if (skill) {
      setName(skill.name);
      setSkillType(skill.skill_type);
      setConfigText(JSON.stringify(skill.config, null, 2));
    } else {
      setName('');
      setSkillType('Tool');
      setConfigText('{}');
    }
  }, [skill, open]);

  if (!open) return null;

  const handleSubmit = () => {
    if (!name.trim()) return;
    let config: Record<string, unknown>;
    try {
      config = JSON.parse(configText);
    } catch {
      config = {};
    }
    onSubmit({ name: name.trim(), skill_type: skillType, config });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
      <div className="bg-white brutal-border brutal-shadow w-full max-w-md">
        {/* Header */}
        <div className="flex items-center justify-between p-3 brutal-border-b bg-brutal-yellow">
          <div className="flex items-center gap-2">
            <Zap size={16} />
            <span className="font-black text-sm uppercase">
              {skill ? 'Edit Skill' : 'Add New Skill'}
            </span>
          </div>
          <button onClick={onClose} className="p-1 brutal-border hover:bg-yellow-400">
            <X size={14} />
          </button>
        </div>

        {/* Form */}
        <div className="p-4 space-y-4">
          {/* Name */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Skill Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Web Search"
              className="w-full brutal-border p-2 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
            />
          </div>

          {/* Type */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Skill Type
            </label>
            <div className="flex gap-1">
              {(['MCP Server', 'Tool', 'Custom Command'] as SkillType[]).map((t) => (
                <button
                  key={t}
                  onClick={() => setSkillType(t)}
                  className={cn(
                    "px-2 py-1 brutal-border text-[10px] font-black uppercase",
                    skillType === t ? "bg-brutal-cyan" : "bg-white hover:bg-gray-100"
                  )}
                >
                  {t}
                </button>
              ))}
            </div>
          </div>

          {/* Config JSON */}
          <div>
            <label className="block text-[10px] font-black uppercase text-gray-500 mb-1">
              Configuration (JSON)
            </label>
            <textarea
              value={configText}
              onChange={(e) => setConfigText(e.target.value)}
              rows={5}
              className="w-full brutal-border p-2 text-xs font-mono focus:outline-none focus:bg-brutal-bg resize-none"
              placeholder='{"key": "value"}'
            />
          </div>

          {/* Actions */}
          <div className="flex items-center justify-between pt-2 brutal-border-t">
            <button
              onClick={onClose}
              className="brutal-btn bg-gray-200 text-xs"
              disabled={submitting}
            >
              Cancel
            </button>
            <button
              onClick={handleSubmit}
              disabled={!name.trim() || submitting}
              className={cn(
                "brutal-btn flex items-center gap-2 text-xs",
                !name.trim() || submitting
                  ? "bg-gray-200 text-gray-400 cursor-not-allowed"
                  : "bg-brutal-pink text-white"
              )}
            >
              <Zap size={12} />
              {submitting ? 'Saving...' : skill ? 'Update Skill' : 'Add Skill'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
