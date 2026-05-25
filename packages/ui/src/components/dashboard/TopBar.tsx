'use client'

import { RefreshCw, Activity, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'

interface Props {
  lastUpdated:  Date
  isRefreshing: boolean
  onRefresh:    () => void
}

export function TopBar({ lastUpdated, isRefreshing, onRefresh }: Props) {
  const timeStr = lastUpdated.toLocaleTimeString([], {
    hour: '2-digit', minute: '2-digit', second: '2-digit'
  })

  return (
    <header className="border-b border-bg-border bg-bg-surface/80 backdrop-blur-sm sticky top-0 z-50">
      <div className="max-w-[1400px] mx-auto px-4 h-14 flex items-center justify-between">

        {/* Left — branding */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <Zap size={16} className="text-accent-green" />
            <span className="font-display text-sm font-bold text-text-primary tracking-widest uppercase">
              StellarFees
            </span>
          </div>
          <span className="text-text-muted text-xs">|</span>
          <span className="text-xs font-mono px-2 py-0.5 rounded border border-accent-cyan/30 text-accent-cyan bg-cyan-950/20">
            TESTNET
          </span>
        </div>

        {/* Center — live indicator */}
        <div className="hidden md:flex items-center gap-2 text-xs text-text-secondary font-mono">
          <Activity size={12} className="text-accent-green" />
          <span>LIVE</span>
          <span className="text-text-muted">·</span>
          <span suppressHydrationWarning>UPDATED {timeStr}</span>
        </div>

        {/* Right — refresh */}
        <button
          onClick={onRefresh}
          className={cn(
            'flex items-center gap-2 text-xs font-mono px-3 py-1.5 rounded',
            'border border-bg-border text-text-secondary',
            'hover:border-accent-green/50 hover:text-accent-green transition-colors',
            isRefreshing && 'opacity-50 cursor-not-allowed'
          )}
          disabled={isRefreshing}
        >
          <RefreshCw
            size={12}
            className={cn(isRefreshing && 'animate-spin')}
          />
          REFRESH
        </button>
      </div>
    </header>
  )
}