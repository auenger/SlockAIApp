export type TabType = 'CHAT' | 'TASKS' | 'WORKSPACE' | 'SKILLS' | 'ACTIVITY' | 'PROFILE';

export interface Agent {
  id: string;
  name: string;
  description: string;
  status: 'online' | 'offline' | 'busy';
  avatar: string;
  color: string;
}

export interface Channel {
  id: string;
  name: string;
  unreadCount?: number;
}

export interface Thread {
  id: string;
  title: string;
  preview: string;
}

export interface Task {
  id: number;
  title: string;
  status: 'TODO' | 'IN PROGRESS' | 'IN REVIEW' | 'DONE';
  assignee?: string;
}

export interface Message {
  id: string;
  sender: {
    name: string;
    avatar: string;
    isAgent: boolean;
  };
  content: string;
  timestamp: string;
  isThinking?: boolean;
}
