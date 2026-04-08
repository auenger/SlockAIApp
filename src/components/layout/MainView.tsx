import { cn } from "../../lib/utils";

interface MainViewProps {
  className?: string;
}

export function MainView({ className }: MainViewProps) {
  return (
    <main
      className={cn(
        "flex-1 flex flex-col bg-slock-bg min-w-0",
        className,
      )}
    >
      <div className="p-4 border-b-3 border-slock-border">
        <h2 className="font-mono text-lg font-semibold">Main View</h2>
      </div>
      <div className="flex-1 flex items-center justify-center">
        <p className="text-slock-text-muted font-mono text-sm">
          Select a channel to start
        </p>
      </div>
    </main>
  );
}
