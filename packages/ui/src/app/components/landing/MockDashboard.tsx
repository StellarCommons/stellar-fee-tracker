'use client'

const BARS = [0.3, 0.5, 0.4, 0.8, 0.6, 0.9, 0.5, 0.7, 0.4, 1.0, 0.6, 0.8, 0.5, 0.3, 0.6, 0.7, 0.4, 0.5, 0.8, 0.6]

const STAT_CARDS = [
  { label: 'BASE FEE', val: '100 str',   color: '#00ff9d' },
  { label: 'AVG FEE',  val: '3,849 str', color: '#e2eaf4' },
  { label: 'STATUS',   val: 'Normal',    color: '#00ff9d' },
  { label: 'SPIKES',   val: '0',         color: '#e2eaf4' },
]

const ROLLING = [
  { l: 'Short-term',  v: '3,849 str', c: '#00ff9d' },
  { l: 'Medium-term', v: '5,234 str', c: '#00d4ff' },
  { l: 'Long-term',   v: '6,102 str', c: '#ffd23f' },
]

export default function MockDashboard() {
  return (
    <div
      style={{ fontFamily: "'JetBrains Mono', monospace" }}
      className="relative rounded-lg overflow-hidden border border-[#1e2a36] bg-[#080b0f] shadow-2xl shadow-black/60 w-full max-w-2xl mx-auto"
    >
      {/* Titlebar */}
      <div className="flex items-center gap-1.5 px-4 py-3 border-b border-[#1e2a36] bg-[#0d1117]">
        <div className="w-3 h-3 rounded-full bg-[#ff5f57]" />
        <div className="w-3 h-3 rounded-full bg-[#febc2e]" />
        <div className="w-3 h-3 rounded-full bg-[#28c840]" />
        <span className="ml-3 text-[10px] text-[#3d5a73] tracking-widest">STELLARFEES — DASHBOARD</span>
        <div className="ml-auto flex items-center gap-1.5">
          <span className="w-1.5 h-1.5 rounded-full bg-[#00ff9d] animate-pulse" />
          <span className="text-[9px] text-[#00ff9d]">LIVE</span>
        </div>
      </div>

      {/* Stat cards */}
      <div className="grid grid-cols-4 gap-px bg-[#1e2a36] border-b border-[#1e2a36]">
        {STAT_CARDS.map((s, i) => (
          <div key={i} className="bg-[#111820] px-3 py-3">
            <div className="text-[8px] text-[#3d5a73] tracking-widest mb-1">{s.label}</div>
            <div className="text-sm font-bold" style={{ color: s.color, fontFamily: "'Space Mono', monospace" }}>
              {s.val}
            </div>
          </div>
        ))}
      </div>

      {/* Chart */}
      <div className="bg-[#0d1117] px-4 pt-3 pb-1">
        <div className="flex items-center justify-between mb-2">
          <span className="text-[8px] text-[#3d5a73] tracking-widest">FEE HISTORY · 1H</span>
          <div className="flex gap-1">
            {['1H', '6H', '24H'].map((w, i) => (
              <span key={w} className="text-[8px] px-1.5 py-0.5 rounded" style={{
                background: i === 0 ? '#00ff9d15' : 'transparent',
                color:      i === 0 ? '#00ff9d'   : '#3d5a73',
                border:     `1px solid ${i === 0 ? '#00ff9d40' : '#1e2a36'}`,
              }}>{w}</span>
            ))}
          </div>
        </div>
        <div className="flex items-end gap-[3px] h-16 pb-1">
          {BARS.map((h, i) => (
            <div key={i} className="flex-1 rounded-sm" style={{
              height:     `${h * 100}%`,
              background: h > 0.85 ? 'linear-gradient(to top, #ff4d6d60, #ff4d6d30)'
                        : h > 0.6  ? 'linear-gradient(to top, #ffd23f60, #ffd23f20)'
                        :             'linear-gradient(to top, #00ff9d60, #00ff9d20)',
              border: `1px solid ${h > 0.85 ? '#ff4d6d30' : h > 0.6 ? '#ffd23f20' : '#00ff9d20'}`,
            }} />
          ))}
        </div>
        <div className="flex justify-between mt-1">
          <span className="text-[7px] text-[#1e2a36]">01:18</span>
          <span className="text-[7px] text-[#1e2a36]">01:38</span>
        </div>
      </div>

      {/* Bottom panels */}
      <div className="grid grid-cols-2 gap-px bg-[#1e2a36]">
        <div className="bg-[#111820] px-3 py-3">
          <div className="text-[8px] text-[#3d5a73] tracking-widest mb-2">TREND ANALYSIS</div>
          <div className="text-sm font-bold text-[#00ff9d] mb-2" style={{ fontFamily: "'Space Mono', monospace" }}>— Normal</div>
          {['1H', '6H', '24H'].map(l => (
            <div key={l} className="flex justify-between py-1 border-b border-[#1e2a3620]">
              <span className="text-[8px] text-[#3d5a73]">{l} change</span>
              <span className="text-[8px] text-[#7a9ab5]">→ 0.0%</span>
            </div>
          ))}
        </div>
        <div className="bg-[#111820] px-3 py-3">
          <div className="text-[8px] text-[#3d5a73] tracking-widest mb-2">ROLLING AVERAGES</div>
          {ROLLING.map(({ l, v, c }) => (
            <div key={l} className="flex justify-between items-center py-1.5 border-b border-[#1e2a3620]">
              <span className="text-[8px] text-[#3d5a73]">{l}</span>
              <span className="text-[9px] font-bold" style={{ color: c, fontFamily: "'Space Mono', monospace" }}>{v}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}