/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { MainContent } from './components/MainContent';
import { ThreadPanel } from './components/ThreadPanel';
import { CreateTaskModal, InviteHumanModal } from './components/Modals';
import { TabType } from './types';

export default function App() {
  const [activeChannel, setActiveChannel] = useState('kagent-integrate-sap-ai-core');
  const [activeTab, setActiveTab] = useState<TabType>('TASKS');
  const [isThreadOpen, setIsThreadOpen] = useState(true);
  
  // Modal states for demonstration
  const [isCreateTaskModalOpen, setIsCreateTaskModalOpen] = useState(false);
  const [isInviteModalOpen, setIsInviteModalOpen] = useState(false);

  return (
    <div className="flex h-screen w-full overflow-hidden bg-brutal-bg">
      {/* Left Sidebar */}
      <Sidebar 
        activeChannel={activeChannel} 
        onChannelSelect={setActiveChannel} 
      />

      {/* Main Content Area */}
      <MainContent 
        activeTab={activeTab} 
        onTabChange={setTab => {
          setActiveTab(setTab);
          // For demo purposes, we can trigger modals based on certain actions
          // In a real app, these would be triggered by buttons
        }} 
      />

      {/* Right Thread Panel */}
      <ThreadPanel 
        isOpen={isThreadOpen} 
        onClose={() => setIsThreadOpen(false)} 
      />

      {/* Modals (Hidden by default, can be toggled for preview) */}
      <CreateTaskModal 
        isOpen={isCreateTaskModalOpen} 
        onClose={() => setIsCreateTaskModalOpen(false)} 
      />
      <InviteHumanModal 
        isOpen={isInviteModalOpen} 
        onClose={() => setIsInviteModalOpen(false)} 
      />

      {/* Floating Demo Controls (Optional, for easy preview of all states) */}
      <div className="fixed bottom-4 right-4 flex flex-col gap-2 z-50">
        <button 
          onClick={() => setIsThreadOpen(!isThreadOpen)}
          className="brutal-btn bg-brutal-cyan text-[10px]"
        >
          Toggle Thread
        </button>
        <button 
          onClick={() => setIsCreateTaskModalOpen(true)}
          className="brutal-btn bg-brutal-pink text-white text-[10px]"
        >
          Demo: Create Task Modal
        </button>
        <button 
          onClick={() => setIsInviteModalOpen(true)}
          className="brutal-btn bg-purple-400 text-white text-[10px]"
        >
          Demo: Invite Modal
        </button>
      </div>
    </div>
  );
}
