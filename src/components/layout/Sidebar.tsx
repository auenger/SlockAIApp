import { cn } from "../../lib/utils";

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className }: SidebarProps) {
  return (
    <aside
      className={cn(
        "w-64 min-w-64 bg-slock-sidebar border-r-3 border-slock-border flex flex-col",
        className,
      )}
    >
      <div className="p-4 border-b-3 border-slock-border">
        <h1 className="font-mono text-xl font-bold tracking-tight">SlockAI</h1>
      </div>
      <nav className="flex-1 p-4 space-y-2 overflow-y-auto">
        <div className="text-sm font-mono text-slock-text-muted uppercase tracking-wider mb-3">
          Channels
        </div>
        <div className="text-slock-text-muted font-mono text-sm p-2">
          No channels yet
        </div>
      </nav>
      <div className="p-4 border-t-3 border-slock-border">
        <div className="text-xs font-mono text-slock-text-muted">
          Agents: None active
        </div>
      </div>
    </aside>
  );
}
