interface StatusDotProps {
  status: 'ok' | 'degraded' | 'down' | 'unknown'
  showLabel?: boolean
}

const STATUS_CONFIG = {
  ok:       { dot: 'bg-emerald-500', label: 'Operational',  text: 'text-emerald-400', animate: true },
  degraded: { dot: 'bg-amber-500',   label: 'Degraded',     text: 'text-amber-400',   animate: true },
  down:     { dot: 'bg-red-500',     label: 'Down',         text: 'text-red-400',     animate: false },
  unknown:  { dot: 'bg-zinc-600',    label: 'Unknown',      text: 'text-zinc-500',    animate: false },
}

export default function StatusDot({ status, showLabel = true }: StatusDotProps) {
  const cfg = STATUS_CONFIG[status]
  return (
    <span className="flex items-center gap-1.5">
      <span
        className={`w-2 h-2 rounded-full flex-shrink-0 ${cfg.dot} ${cfg.animate ? 'animate-pulse' : ''}`}
      />
      {showLabel && (
        <span className={`text-xs ${cfg.text}`}>{cfg.label}</span>
      )}
    </span>
  )
}
