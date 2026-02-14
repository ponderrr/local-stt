interface StatusIndicatorProps {
  status: 'idle' | 'listening' | 'loading' | 'error';
  hotkey: string;
}

const statusConfig = {
  idle: { color: 'bg-zinc-600', label: 'Idle', pulse: false },
  listening: { color: 'bg-emerald-500', label: 'Listening', pulse: true },
  loading: { color: 'bg-amber-500', label: 'Loading Model...', pulse: true },
  error: { color: 'bg-red-500', label: 'Error', pulse: false },
} as const;

export function StatusIndicator({ status, hotkey }: StatusIndicatorProps) {
  const config = statusConfig[status];

  return (
    <div className="flex items-center gap-3">
      <div className="flex items-center gap-2">
        <div
          className={`w-2 h-2 rounded-full ${config.color} ${config.pulse ? 'animate-pulse' : ''}`}
        />
        <span className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
          {config.label}
        </span>
      </div>
      <span className="text-xs text-zinc-600 font-mono">[{hotkey}]</span>
    </div>
  );
}
