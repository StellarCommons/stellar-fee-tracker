'use client'
import React from "react";

const FEATURES = [
  { icon: '⚡', label: 'Live Polling',          desc: 'Fetches fee stats from Horizon every 10 seconds',         angle: 0   },
  { icon: '📊', label: 'Percentile Breakdown',  desc: 'P10 through P99 distribution on every refresh',           angle: 72  },
  { icon: '📈', label: 'Trend Detection',       desc: 'Rising, Normal, Declining, Congested status',             angle: 144 },
  { icon: '🗄️', label: 'SQLite Persistence',   desc: 'Fee history stored locally, queryable by window',         angle: 216 },
  { icon: '🔔', label: 'Spike Alerts',          desc: 'Webhook delivery when fees spike above threshold',        angle: 288 },
]

export function FeatureOrbit() {
  return (
    <section className="relative z-10 py-24 px-6 border-t border-[#1e2a36]">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <div className="text-[10px] text-[#00ff9d] tracking-widest mb-3">CAPABILITIES</div>
          <h2
            className="text-3xl md:text-4xl font-bold text-[#e2eaf4]"
            style={{ fontFamily: "'Space Mono', monospace" }}
          >
            Built for developers.
          </h2>
        </div>

        {/* Orbit */}
        <div className="relative w-full max-w-2xl mx-auto" style={{ height: '460px' }}>
          {/* Center orb */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-10">
            <div className="relative w-24 h-24 rounded-full border border-[#00ff9d30] bg-[#0d1117] flex items-center justify-center">
              <div
                className="absolute inset-0 rounded-full border border-[#00ff9d20] animate-ping"
                style={{ animationDuration: '3s' }}
              />
              <span
                className="text-[#00ff9d] text-xs font-bold tracking-widest text-center leading-tight"
                style={{ fontFamily: "'Space Mono', monospace" }}
              >
                STELLAR<br />FEES
              </span>
            </div>
          </div>

          {/* Orbit rings */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-72 h-72 rounded-full border border-dashed border-[#1e2a36]" />
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 rounded-full border border-[#0d1117]" />

          {/* Feature nodes */}
          {FEATURES.map(({ icon, label, desc, angle }) => {
            const rad = (angle - 90) * (Math.PI / 180)
            const r = 144
            const x = 50 + (r / 4.6) * Math.cos(rad)
            const y = 50 + (r / 4.6) * Math.sin(rad)

            return (
              <div
                key={label}
                className="absolute group"
                style={{ left: `${x}%`, top: `${y}%`, transform: 'translate(-50%, -50%)' }}
              >
                {/* Connector line */}
                <div
                  className="absolute top-1/2 left-1/2 w-px bg-gradient-to-b from-[#00ff9d20] to-transparent"
                  style={{
                    height: `${r * 0.62}px`,
                    transformOrigin: 'top center',
                    transform: `rotate(${angle + 180}deg)`,
                    opacity: 0.4,
                  }}
                />
                {/* Node card */}
                <div className="relative z-10 w-28 bg-[#111820] border border-[#1e2a36] rounded-lg p-3 cursor-default
                  transition-all duration-300
                  group-hover:border-[#00ff9d40] group-hover:bg-[#0d2218]
                  group-hover:-translate-y-1 group-hover:shadow-lg group-hover:shadow-[#00ff9d08]">
                  <div className="text-lg mb-1">{icon}</div>
                  <div
                    className="text-[10px] text-[#e2eaf4] font-bold leading-tight mb-1"
                    style={{ fontFamily: "'Space Mono', monospace" }}
                  >
                    {label}
                  </div>
                  <div className="text-[9px] text-[#3d5a73] leading-tight hidden group-hover:block">
                    {desc}
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      </div>
    </section>
  )
}