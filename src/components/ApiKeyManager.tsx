/**
 * API Key Management Modal.
 *
 * Displays a list of API keys (masked), allows adding new keys
 * and deleting existing ones. Keys are stored securely in the OS keyring.
 */

import React, { useState, useEffect } from 'react';
import { X, Key, Plus, Trash2, Check, AlertCircle, Loader2, User } from 'lucide-react';
import { cn } from '../lib/utils';
import { useApiKeys } from '../lib/useApiKeys';
import { useUserProfile } from '../lib/useUserProfile';

// ---------------------------------------------------------------------------
// Provider definitions
// ---------------------------------------------------------------------------

interface ProviderOption {
  id: string;
  name: string;
}

const PROVIDERS: ProviderOption[] = [
  { id: 'claude-code', name: 'Claude Code' },
  { id: 'openai', name: 'OpenAI' },
  { id: 'anthropic', name: 'Anthropic' },
  { id: 'gemini', name: 'Gemini' },
];

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface ApiKeyManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const ApiKeyManager: React.FC<ApiKeyManagerProps> = ({ isOpen, onClose }) => {
  const { keys, loading, error, loadKeys, addKey, removeKey, clearError } = useApiKeys();
  const { profile, updateProfile } = useUserProfile();
  const [profileName, setProfileName] = useState(profile.name);
  const [profileEmail, setProfileEmail] = useState(profile.email);
  const [showAddForm, setShowAddForm] = useState(false);
  const [selectedProvider, setSelectedProvider] = useState<string>(PROVIDERS[0].id);
  const [newKeyValue, setNewKeyValue] = useState('');
  const [adding, setAdding] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  // Load keys when modal opens
  useEffect(() => {
    if (isOpen) {
      loadKeys();
      setShowAddForm(false);
      setNewKeyValue('');
      setDeleteConfirm(null);
      setSuccessMsg(null);
      clearError();
      setProfileName(profile.name);
      setProfileEmail(profile.email);
    }
  }, [isOpen, loadKeys, clearError, profile]);

  /** Handle adding a new key */
  const handleAdd = async () => {
    if (!newKeyValue.trim()) return;
    setAdding(true);
    clearError();
    try {
      await addKey(selectedProvider, newKeyValue.trim());
      setNewKeyValue('');
      setShowAddForm(false);
      setSuccessMsg(`Key for ${PROVIDERS.find(p => p.id === selectedProvider)?.name} added successfully`);
      setTimeout(() => setSuccessMsg(null), 3000);
    } catch {
      // error is set by the hook
    } finally {
      setAdding(false);
    }
  };

  /** Handle deleting a key */
  const handleDelete = async (runtimeId: string) => {
    setDeleting(true);
    clearError();
    try {
      await removeKey(runtimeId);
      setDeleteConfirm(null);
      setSuccessMsg(`Key for ${PROVIDERS.find(p => p.id === runtimeId)?.name || runtimeId} deleted`);
      setTimeout(() => setSuccessMsg(null), 3000);
    } catch {
      // error is set by the hook
    } finally {
      setDeleting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />

      {/* Modal */}
      <div className="relative w-full max-w-lg mx-4 bg-white brutal-shadow brutal-border flex flex-col max-h-[80vh]">
        {/* Header */}
        <div className="flex items-center justify-between p-4 brutal-border-b bg-black text-white">
          <div className="flex items-center gap-2">
            <Key size={20} />
            <h2 className="font-black text-lg">API Key Management</h2>
          </div>
          <button onClick={onClose} className="p-1 hover:bg-white/20 transition-colors">
            <X size={20} />
          </button>
        </div>

        {/* Success message */}
        {successMsg && (
          <div className="mx-4 mt-3 flex items-center gap-2 p-2 bg-brutal-green/20 brutal-border text-xs font-bold text-green-700">
            <Check size={14} />
            {successMsg}
          </div>
        )}

        {/* Error message */}
        {error && (
          <div className="mx-4 mt-3 flex items-center gap-2 p-2 bg-brutal-pink/20 brutal-border text-xs font-bold text-red-700">
            <AlertCircle size={14} />
            {error}
          </div>
        )}

        {/* User Profile Section */}
        <div className="p-4 brutal-border-b bg-gray-50 space-y-3">
          <div className="flex items-center gap-2">
            <User size={16} />
            <span className="text-xs font-black uppercase">Your Profile</span>
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={profileName}
              onChange={(e) => setProfileName(e.target.value)}
              placeholder="Your name"
              className="flex-1 brutal-border px-2 py-1.5 text-xs font-bold bg-white focus:outline-none focus:bg-brutal-bg"
              onBlur={() => updateProfile({ name: profileName })}
              onKeyDown={(e) => {
                if (e.key === 'Enter') updateProfile({ name: profileName });
              }}
            />
            <input
              type="email"
              value={profileEmail}
              onChange={(e) => setProfileEmail(e.target.value)}
              placeholder="Email"
              className="flex-1 brutal-border px-2 py-1.5 text-xs font-bold bg-white focus:outline-none focus:bg-brutal-bg"
              onBlur={() => updateProfile({ email: profileEmail })}
              onKeyDown={(e) => {
                if (e.key === 'Enter') updateProfile({ email: profileEmail });
              }}
            />
          </div>
        </div>

        {/* Key List */}
        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {loading && keys.length === 0 ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 size={24} className="animate-spin text-gray-400" />
            </div>
          ) : (
            keys.map((keyInfo) => (
              <div
                key={keyInfo.id}
                className={cn(
                  "brutal-card flex items-center justify-between py-3 px-4",
                  !keyInfo.has_key && "opacity-60"
                )}
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className={cn(
                    "w-8 h-8 brutal-border flex items-center justify-center shrink-0",
                    keyInfo.has_key ? "bg-brutal-green" : "bg-gray-200"
                  )}>
                    <Key size={16} />
                  </div>
                  <div className="min-w-0">
                    <div className="font-black text-sm">{keyInfo.name}</div>
                    {keyInfo.has_key ? (
                      <div className="text-[10px] font-mono text-gray-500 truncate">
                        {keyInfo.masked_key}
                      </div>
                    ) : (
                      <div className="text-[10px] text-gray-400 italic">No key configured</div>
                    )}
                  </div>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  {keyInfo.has_key && (
                    <>
                      {deleteConfirm === keyInfo.id ? (
                        <>
                          <button
                            onClick={() => handleDelete(keyInfo.id)}
                            disabled={deleting}
                            className="px-2 py-1 brutal-border bg-brutal-pink text-white text-[10px] font-black hover:bg-pink-600"
                          >
                            {deleting ? 'Deleting...' : 'Confirm'}
                          </button>
                          <button
                            onClick={() => setDeleteConfirm(null)}
                            className="px-2 py-1 brutal-border bg-white text-[10px] font-black hover:bg-gray-100"
                          >
                            Cancel
                          </button>
                        </>
                      ) : (
                        <button
                          onClick={() => setDeleteConfirm(keyInfo.id)}
                          className="p-1.5 brutal-border hover:bg-gray-100"
                          title="Delete key"
                        >
                          <Trash2 size={14} />
                        </button>
                      )}
                    </>
                  )}
                </div>
              </div>
            ))
          )}
        </div>

        {/* Add Key Form */}
        <div className="p-4 brutal-border-t bg-gray-50">
          {showAddForm ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-black uppercase">Add New Key</span>
                <button
                  onClick={() => { setShowAddForm(false); setNewKeyValue(''); }}
                  className="p-0.5 hover:bg-gray-200"
                >
                  <X size={14} />
                </button>
              </div>
              <div className="flex gap-2">
                <select
                  value={selectedProvider}
                  onChange={(e) => setSelectedProvider(e.target.value)}
                  className="brutal-border px-2 py-1.5 text-xs font-bold bg-white focus:outline-none focus:bg-brutal-bg"
                >
                  {PROVIDERS.map((p) => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </select>
                <input
                  type="password"
                  value={newKeyValue}
                  onChange={(e) => setNewKeyValue(e.target.value)}
                  placeholder="Enter API key..."
                  className="flex-1 brutal-border px-2 py-1.5 text-xs font-mono focus:outline-none focus:bg-brutal-bg"
                  autoFocus
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleAdd();
                  }}
                />
              </div>
              <button
                onClick={handleAdd}
                disabled={!newKeyValue.trim() || adding}
                className={cn(
                  "w-full brutal-btn text-xs font-black flex items-center justify-center gap-2",
                  newKeyValue.trim() && !adding
                    ? "bg-brutal-pink text-white"
                    : "bg-gray-200 text-gray-400 cursor-not-allowed"
                )}
              >
                {adding ? (
                  <>
                    <Loader2 size={14} className="animate-spin" />
                    Adding...
                  </>
                ) : (
                  <>
                    <Check size={14} />
                    Save Key
                  </>
                )}
              </button>
            </div>
          ) : (
            <button
              onClick={() => setShowAddForm(true)}
              className="w-full brutal-btn bg-brutal-cyan text-xs font-black flex items-center justify-center gap-2"
            >
              <Plus size={14} />
              Add API Key
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
