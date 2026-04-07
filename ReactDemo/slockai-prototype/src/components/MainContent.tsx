import React, { useState, useRef, useEffect } from 'react';
import { 
  MessageSquare, 
  CheckSquare, 
  Folder, 
  Zap, 
  Activity, 
  User,
  Plus,
  Filter,
  Trash2,
  Users,
  ChevronRight,
  Bot,
  Circle,
  Square,
  RotateCcw,
  Copy,
  Send,
  Image as ImageIcon,
  Terminal,
  Search,
  Code,
  Globe,
  Database
} from 'lucide-react';
import { cn } from '../lib/utils';
import { TabType, Task, Message } from '../types';
import { generateAgentResponse } from '../services/geminiService';

interface MainContentProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
}

export const MainContent: React.FC<MainContentProps> = ({ activeTab, onTabChange }) => {
  const [taskFilter, setTaskFilter] = useState('All');
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isThinking, setIsThinking] = useState(false);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [logs, setLogs] = useState<{ time: string; status: string; text?: string; color?: string }[]>([
    { time: '08:27:17 AM', status: 'Thinking', color: 'text-brutal-yellow' },
    { time: '08:27:17 AM', status: 'Idle', color: 'text-brutal-green', text: 'Idle' },
    { time: '08:30:09 AM', status: 'Working', color: 'text-brutal-cyan', text: 'Message received' },
  ]);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const addLog = (status: string, text?: string, color?: string) => {
    const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    setLogs(prev => [...prev, { time, status, text, color }]);
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, isThinking]);

  const handleSendMessage = async () => {
    if (!inputValue.trim()) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      sender: {
        name: 'Lissa',
        avatar: 'L',
        isAgent: false,
      },
      content: inputValue,
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    };

    setMessages(prev => [...prev, userMessage]);
    setInputValue('');
    setIsThinking(true);
    addLog('Working', 'Message received from Lissa', 'text-brutal-cyan');
    addLog('Thinking', undefined, 'text-brutal-yellow');

    try {
      const history = messages.map(m => ({
        role: m.sender.isAgent ? 'model' as const : 'user' as const,
        parts: m.content
      }));

      const response = await generateAgentResponse(
        '克劳德',
        '你是一个非常资深的软件开发工程师，负责软件的架构设计和开发',
        inputValue,
        history
      );

      const agentMessage: Message = {
        id: (Date.now() + 1).toString(),
        sender: {
          name: '克劳德',
          avatar: 'C',
          isAgent: true,
        },
        content: response || 'Sorry, I encountered an error.',
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      };

      setMessages(prev => [...prev, agentMessage]);
      addLog('Output', `Responded to: ${inputValue.substring(0, 20)}...`);
    } catch (error) {
      console.error("Agent Error:", error);
      addLog('Error', 'Failed to generate response', 'text-brutal-pink');
    } finally {
      setIsThinking(false);
      addLog('Idle', 'Task completed', 'text-brutal-green');
    }
  };

  const tabs: { id: TabType; label: string; icon: any }[] = [
    { id: 'CHAT', label: 'CHAT', icon: MessageSquare },
    { id: 'TASKS', label: 'TASKS', icon: CheckSquare },
    { id: 'WORKSPACE', label: 'WORKSPACE', icon: Folder },
    { id: 'SKILLS', label: 'SKILLS', icon: Zap },
    { id: 'ACTIVITY', label: 'ACTIVITY', icon: Activity },
    { id: 'PROFILE', label: 'PROFILE', icon: User },
  ];

  const tasks: Task[] = [
    { id: 2, title: 'test', status: 'TODO' },
    { id: 1, title: '总结oauth2在kagent中的流程', status: 'IN PROGRESS', assignee: '克劳德' },
  ];

  return (
    <div className="flex-1 flex flex-col min-w-0 bg-white">
      {/* Top Header */}
      <div className="p-3 brutal-border-b flex items-center justify-between bg-white">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 brutal-border bg-brutal-cyan flex items-center justify-center">
            <Bot size={24} />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="font-black text-lg">克劳德</h2>
              <span className="text-[10px] text-gray-500 font-medium">你是一个非常资深的软件开发工程师...</span>
            </div>
            <div className="flex items-center gap-1.5">
              <Circle size={8} fill="#39FF14" className="text-brutal-green" />
              <span className="text-[10px] font-bold uppercase tracking-tighter">Online</span>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button className="p-2 brutal-border hover:bg-gray-100">
            <Square size={18} />
          </button>
          <button className="p-2 brutal-border hover:bg-gray-100">
            <RotateCcw size={18} />
          </button>
          <button className="p-2 brutal-border bg-brutal-yellow hover:bg-yellow-400">
            <Trash2 size={18} />
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex brutal-border-b bg-gray-50">
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={cn(
              "px-4 py-2 font-black text-xs flex items-center gap-2 brutal-border-r transition-all",
              activeTab === tab.id ? "bg-brutal-yellow brutal-border-b-0 translate-y-[1px]" : "hover:bg-gray-200"
            )}
          >
            <tab.icon size={14} />
            {tab.label}
          </button>
        ))}
        <div className="flex-1" />
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-y-auto p-4 bg-brutal-bg/20">
        {activeTab === 'TASKS' && (
          <div className="space-y-4">
            {/* Task Filters */}
            <div className="flex items-center justify-between">
              <div className="flex gap-1">
                {['All 2', 'Todo 1', 'In Progress 1', 'In Review', 'Done'].map(f => (
                  <button
                    key={f}
                    onClick={() => setTaskFilter(f.split(' ')[0])}
                    className={cn(
                      "px-2 py-0.5 brutal-border text-[10px] font-black uppercase",
                      taskFilter === f.split(' ')[0] ? "bg-brutal-yellow" : "bg-white hover:bg-gray-100"
                    )}
                  >
                    {f}
                  </button>
                ))}
              </div>
              <button className="brutal-btn bg-brutal-pink text-white text-xs flex items-center gap-1">
                <Plus size={14} /> New Task
              </button>
            </div>

            {/* Task List */}
            <div className="space-y-2">
              {tasks.map(task => (
                <div key={task.id} className="brutal-card flex items-center justify-between py-2 px-3">
                  <div className="flex items-center gap-3">
                    <ChevronRight size={14} className="text-gray-400" />
                    <span className="text-gray-400 font-mono text-xs">#{task.id}</span>
                    <span className={cn(
                      "px-1.5 py-0.5 brutal-border text-[8px] font-black",
                      task.status === 'TODO' ? "bg-brutal-yellow" : "bg-brutal-cyan"
                    )}>
                      {task.status}
                    </span>
                    <span className="font-bold text-sm">{task.title}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {task.assignee && (
                      <div className="flex items-center gap-1 px-1.5 py-0.5 brutal-border bg-gray-50 text-[10px] font-bold">
                        <span className="text-gray-500">@</span>{task.assignee}
                      </div>
                    )}
                    <div className="flex gap-1">
                      <button className="p-1 brutal-border bg-brutal-cyan hover:bg-cyan-400">
                        <Users size={14} />
                      </button>
                      <button className="p-1 brutal-border hover:bg-gray-100">
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'WORKSPACE' && (
          <div className="h-full flex flex-col">
            <div className="flex items-center gap-2 mb-4 text-[10px] font-mono text-gray-500 bg-white brutal-border p-1">
              <span>~/.slock/agents/8ce6fe04-742e-4e16-bfa2-0ec0a48b2c06/</span>
              <button className="p-0.5 brutal-border hover:bg-gray-100">
                <Copy size={10} />
              </button>
            </div>
            <div className="flex-1 flex brutal-border bg-white overflow-hidden">
              <div className="w-64 brutal-border-r p-2 space-y-1 overflow-y-auto">
                <div className="flex items-center justify-between px-1 mb-2">
                  <span className="text-[10px] font-black uppercase text-gray-500">Workspace</span>
                  <RotateCcw size={12} className="text-gray-400" />
                </div>
                <div className="space-y-1">
                  {['kagent', 'kubectl-mcp-server', 'notes'].map(dir => (
                    <div key={dir} className="flex items-center gap-2 px-2 py-1 text-sm font-bold hover:bg-gray-100 cursor-pointer">
                      <ChevronRight size={14} className="text-gray-400" />
                      <Folder size={14} className="text-brutal-yellow fill-brutal-yellow" />
                      {dir}
                    </div>
                  ))}
                  <div 
                    onClick={() => setSelectedFile('MEMORY.md')}
                    className={cn(
                      "flex items-center gap-2 px-2 py-1 text-sm font-bold cursor-pointer transition-colors",
                      selectedFile === 'MEMORY.md' ? "bg-brutal-bg brutal-border-l-4 border-l-black" : "hover:bg-gray-100"
                    )}
                  >
                    <MessageSquare size={14} className="text-gray-500" />
                    MEMORY.md
                  </div>
                </div>
              </div>
              <div className="flex-1 flex flex-col p-4 overflow-y-auto bg-gray-50">
                {selectedFile === 'MEMORY.md' ? (
                  <div className="w-full h-full font-mono text-xs space-y-4">
                    <div className="flex items-center justify-between border-b pb-2">
                      <span className="font-black">MEMORY.md</span>
                      <span className="text-gray-400">Markdown • 1.2KB</span>
                    </div>
                    <div className="prose prose-sm max-w-none">
                      <h1 className="font-black text-lg">Agent Memory</h1>
                      <p>This file contains persistent context for the agent.</p>
                      <h2 className="font-bold mt-4">Current Context:</h2>
                      <ul className="list-disc pl-4 space-y-1">
                        <li>Project: KAgent Integration</li>
                        <li>Platform: SAP AI Core</li>
                        <li>Status: Architecture Review</li>
                      </ul>
                      <div className="mt-6 p-3 bg-black text-white brutal-border">
                        <code className="text-[10px]">
                          last_sync: 2026-04-07T02:30:00Z<br/>
                          active_threads: 2
                        </code>
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="flex-1 flex flex-col items-center justify-center text-gray-400">
                    <Folder size={48} strokeWidth={1} className="mb-2 opacity-20" />
                    <span className="text-xs italic">Select a file to view</span>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'CHAT' && (
          <div className="h-full flex flex-col">
             <div className="flex-1 overflow-y-auto space-y-6 pb-4">
                {messages.length === 0 && !isThinking && (
                  <div className="h-full flex flex-col justify-center items-center text-gray-400 italic text-sm">
                    No messages yet. Start a conversation by @mentioning an agent.
                  </div>
                )}
                
                {messages.map((msg) => (
                  <div key={msg.id} className="flex gap-3 px-2">
                    <div className={cn(
                      "w-8 h-8 brutal-border flex items-center justify-center shrink-0 font-black",
                      msg.sender.isAgent ? "bg-brutal-cyan" : "bg-purple-400"
                    )}>
                      {msg.sender.isAgent ? <Bot size={18} /> : <User size={18} />}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">{msg.sender.name}</span>
                        <span className="text-[8px] text-gray-500 uppercase">{msg.timestamp}</span>
                      </div>
                      <div className="text-sm leading-relaxed whitespace-pre-wrap">
                        {msg.content}
                      </div>
                    </div>
                  </div>
                ))}

                {isThinking && (
                  <div className="flex gap-3 px-2 animate-pulse">
                    <div className="w-8 h-8 brutal-border bg-brutal-cyan flex items-center justify-center shrink-0">
                      <Bot size={18} />
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-black text-xs">克劳德</span>
                        <span className="text-[8px] text-gray-500 uppercase italic">Thinking...</span>
                      </div>
                      <div className="h-4 bg-gray-200 w-2/3 brutal-border-b" />
                    </div>
                  </div>
                )}
                <div ref={messagesEndRef} />
             </div>

             <div className="mt-4">
                <div className="relative">
                  <textarea
                    value={inputValue}
                    onChange={(e) => setInputValue(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleSendMessage();
                      }
                    }}
                    placeholder="Message #kagent-integrate-sap-ai-core"
                    className="w-full brutal-border bg-white p-3 min-h-[100px] text-sm focus:outline-none focus:bg-brutal-bg resize-none"
                  />
                </div>
                <div className="flex items-center justify-between mt-2">
                  <div className="flex gap-1">
                    <button className="p-2 brutal-border hover:bg-gray-100">
                      <Plus size={20} />
                    </button>
                    <button className="p-2 brutal-border hover:bg-gray-100">
                      <ImageIcon size={20} />
                    </button>
                  </div>
                  <div className="flex items-center gap-4">
                    <label className="flex items-center gap-2 text-xs font-bold cursor-pointer">
                      <input type="checkbox" className="brutal-border w-4 h-4 accent-black" />
                      As Task
                    </label>
                    <button 
                      onClick={handleSendMessage}
                      disabled={!inputValue.trim() || isThinking}
                      className={cn(
                        "brutal-btn flex items-center gap-2",
                        (!inputValue.trim() || isThinking) ? "bg-gray-200 text-gray-400 cursor-not-allowed" : "bg-brutal-pink text-white"
                      )}
                    >
                      Send <Send size={16} />
                    </button>
                  </div>
                </div>
             </div>
          </div>
        )}

        {activeTab === 'SKILLS' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {[
                { name: 'Terminal Access', icon: Terminal, desc: 'Execute commands in the workspace environment.', status: 'Active', color: 'bg-brutal-green' },
                { name: 'Web Search', icon: Globe, desc: 'Browse the web for real-time information.', status: 'Active', color: 'bg-brutal-cyan' },
                { name: 'Code Analysis', icon: Code, desc: 'Deep understanding of codebase and dependencies.', status: 'Active', color: 'bg-brutal-pink' },
                { name: 'Database Query', icon: Database, desc: 'Interact with connected databases.', status: 'Inactive', color: 'bg-gray-300' },
                { name: 'Knowledge Base', icon: Search, desc: 'Search through internal documentation.', status: 'Active', color: 'bg-brutal-yellow' },
              ].map((skill, i) => (
                <div key={i} className="brutal-card group hover:translate-x-[-2px] hover:translate-y-[-2px] transition-all">
                  <div className="flex items-start justify-between mb-3">
                    <div className={cn("p-2 brutal-border", skill.color)}>
                      <skill.icon size={24} />
                    </div>
                    <span className={cn(
                      "text-[8px] font-black px-1.5 py-0.5 brutal-border uppercase",
                      skill.status === 'Active' ? "bg-brutal-green" : "bg-gray-200"
                    )}>
                      {skill.status}
                    </span>
                  </div>
                  <h4 className="font-black text-sm mb-1">{skill.name}</h4>
                  <p className="text-xs text-gray-600 leading-tight">{skill.desc}</p>
                  <div className="mt-4 flex justify-end">
                    <button className="text-[10px] font-black uppercase underline hover:text-brutal-pink">Configure</button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'ACTIVITY' && (
          <div className="space-y-1 font-mono text-[11px]">
            {logs.map((log, i) => (
              <div key={i} className="flex gap-4 py-1 hover:bg-white/50 px-2 transition-colors">
                <span className="text-gray-400 shrink-0">{log.time}</span>
                <div className="flex items-center gap-2 shrink-0 w-24">
                  <Circle 
                    size={8} 
                    fill={
                      log.status === 'Thinking' ? '#FFDE00' : 
                      log.status === 'Idle' ? '#39FF14' : 
                      log.status === 'Working' ? '#00E5FF' : 
                      log.status === 'Error' ? '#FF4081' : '#000000'
                    } 
                  />
                  <span className={cn("font-bold", log.color)}>{log.status}</span>
                </div>
                <span className="text-gray-600 italic truncate">{log.text}</span>
              </div>
            ))}
          </div>
        )}

        {activeTab === 'PROFILE' && (
          <div className="space-y-6">
            <div className="flex flex-col items-center py-8 brutal-border bg-white brutal-shadow-sm">
              <div className="w-20 h-20 brutal-border bg-brutal-cyan flex items-center justify-center mb-4">
                <Bot size={48} />
              </div>
              <h2 className="text-2xl font-black">克劳德</h2>
              <span className="text-gray-500 font-mono text-xs">@克劳德</span>
            </div>

            <section className="space-y-2">
              <div className="flex items-center justify-between">
                <h3 className="font-black text-xs uppercase tracking-widest text-gray-500">Role</h3>
                <button className="p-1 hover:bg-gray-200 rounded"><Plus size={14}/></button>
              </div>
              <div className="brutal-card text-sm leading-relaxed">
                你是一个非常资深的软件开发工程师，负责软件的架构设计和开发
              </div>
            </section>

            <section className="space-y-2">
              <h3 className="font-black text-xs uppercase tracking-widest text-gray-500">Configuration</h3>
              <div className="brutal-card grid grid-cols-2 gap-6">
                <div>
                  <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Runtime</label>
                  <div className="brutal-btn bg-brutal-cyan text-xs inline-block">Claude Code</div>
                </div>
                <div>
                  <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Model</label>
                  <div className="brutal-btn bg-purple-300 text-xs inline-block">Opus</div>
                </div>
                <div>
                  <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Machine</label>
                  <div className="text-xs font-mono font-bold">LQJF7PWH66 <Circle size={8} fill="#39FF14" className="inline ml-1" /> <span className="text-[10px] text-gray-500">Connected</span></div>
                </div>
                <div>
                  <label className="block text-[10px] font-black text-gray-400 uppercase mb-1">Created</label>
                  <div className="text-xs font-mono font-bold">Apr 3, 2026</div>
                </div>
              </div>
            </section>
          </div>
        )}
      </div>
    </div>
  );
};
