import React from 'react';
import { X, Plus, Trash2 } from 'lucide-react';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
}

export const Modal: React.FC<ModalProps> = ({ isOpen, onClose, title, children, footer }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md brutal-card brutal-shadow p-0 overflow-hidden">
        <div className="p-4 brutal-border-b bg-gray-50 flex items-center justify-between">
          <h3 className="font-black text-sm uppercase tracking-widest">{title}</h3>
          <button onClick={onClose} className="p-1 brutal-border hover:bg-gray-200">
            <X size={16} />
          </button>
        </div>
        <div className="p-6">
          {children}
        </div>
        {footer && (
          <div className="p-4 brutal-border-t bg-gray-50 flex justify-end gap-3">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
};

export const CreateTaskModal: React.FC<{ isOpen: boolean; onClose: () => void }> = ({ isOpen, onClose }) => {
  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Create Tasks"
      footer={
        <>
          <button onClick={onClose} className="brutal-btn bg-white">Cancel</button>
          <button className="brutal-btn bg-brutal-pink text-white">Create 1 Tasks</button>
        </>
      }
    >
      <div className="space-y-4">
        <div className="flex gap-2">
          <input 
            type="text" 
            defaultValue="总结oauth2在kagent中的流程"
            className="flex-1 brutal-border p-2 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
          />
          <button className="p-2 brutal-border hover:bg-gray-100">
            <Trash2 size={18} />
          </button>
        </div>
        <div className="flex gap-2">
          <input 
            type="text" 
            placeholder="Task 2"
            className="flex-1 brutal-border p-2 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
          />
          <button className="p-2 brutal-border hover:bg-gray-100">
            <Trash2 size={18} />
          </button>
        </div>
        <button className="flex items-center gap-2 text-xs font-black uppercase brutal-border p-1.5 hover:bg-gray-100">
          <Plus size={14} /> Add another
        </button>
      </div>
    </Modal>
  );
};

export const InviteHumanModal: React.FC<{ isOpen: boolean; onClose: () => void }> = ({ isOpen, onClose }) => {
  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Invite Human"
      footer={
        <>
          <button onClick={onClose} className="brutal-btn bg-white">Cancel</button>
          <button className="brutal-btn bg-brutal-pink text-white flex items-center gap-2 italic">
            Invite Human
          </button>
        </>
      }
    >
      <div className="space-y-2">
        <label className="block text-[10px] font-black uppercase text-gray-500">
          Email <span className="text-brutal-pink">*</span>
        </label>
        <input 
          type="email" 
          placeholder="user@example.com"
          className="w-full brutal-border p-3 text-sm font-bold focus:outline-none focus:bg-brutal-bg"
        />
      </div>
    </Modal>
  );
};
