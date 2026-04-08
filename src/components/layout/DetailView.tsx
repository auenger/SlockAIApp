import { cn } from "../../lib/utils";

interface DetailViewProps {
  className?: string;
}

export function DetailView({ className }: DetailViewProps) {
  return (
    <aside
      className={cn(
        "w-80 min-w-80 bg-slock-bg border-l-3 border-slock-border flex flex-col",
        className,
      )}
    >
      <div className="p-4 border-b-3 border-slock-border">
        <h2 className="font-mono text-lg font-semibold">Detail</h2>
      </div>
      <div className="flex-1 flex items-center justify-center">
        <p className="text-slock-text-muted font-mono text-sm">
          No thread selected
        </p>
      </div>
    </aside>
  );
}
