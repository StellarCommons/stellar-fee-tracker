'use client'
import React from "react";

const STEPS = [
  {
    step:   '01',
    title:  'Horizon API',
    cmd:    'GET https://horizon-testnet.stellar.org/fee_stats',
    desc:   "Polls Stellar's Horizon API every 10 seconds, fetching live fee statistics including min, max, mode and full percentile distribution.",
    color:  '#00ff9d',
    output: '{ "fee_charged": { "min": "100", "mode": "3849", "p95": "61684" ... } }',
  },
  {
    step:   '02',
    title:  'Rust Backend',
    cmd:    'cargo run -- --network testnet',
    desc:   'Axum-powered REST API processes, stores and analyses fee data. Insights engine computes rolling averages, detects spikes and trends.',
    color:  '#00d4ff',
    output: 'INFO Insights updated — 10000 points · short-term avg: 3849.0 stroops',
  },
  {
    step:   '03',
    title:  'Dashboard',
    cmd:    'npm run dev',
    desc:   'Next.js frontend polls the backend every 10 seconds and renders live data across all panels — no page refresh needed.',
    color:  '#ffd23f',
    output: '✓ LIVE · UPDATED 01:38:45 · 10,000 transactions tracked',
  },
]

export function HowItWorks() {
  return (
    <section className="relative z-10 py-24 px-6 border-t border-[#1e2a36]">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-16">
          <div className="text-[10px] text-[#00ff9d] tracking-widest mb-3">ARCHITECTURE</div>
          <h2
            className="text-3xl md:text-4xl font-bold text-[#e2eaf4]"
            style={{ fontFamily: "'Space Mono', monospace" }}
          >
            How it works.
          </h2>
        </div>

        {/* Steps */}
        <div className="relative">
          {/* Vertical line */}
          <div className="absolute left-6 top-0 bottom-0 w-px bg-gradient-to-b from-[#00ff9d40] via-[#00d4ff40] to-transparent" />

          {STEPS.map(({ step, title, cmd, desc, color, output }) => (
            <div key={step} className="relative pl-16 pb-14 last:pb-0">
              {/* Dot */}
              <div
                className="absolute left-4 top-1 w-5 h-5 rounded-full border-2 flex items-center justify-center"
                style={{ borderColor: color, background: '#080b0f' }}
              >
                <div className="w-1.5 h-1.5 rounded-full" style={{ background: color }} />
              </div>

              <div
                className="text-[9px] tracking-widest mb-1"
                style={{ color, fontFamily: "'Space Mono', monospace" }}
              >
                STEP {step}
              </div>
              <h3
                className="text-xl font-bold text-[#e2eaf4] mb-2"
                style={{ fontFamily: "'Space Mono', monospace" }}
              >
                {title}
              </h3>
              <p className="text-sm text-[#7a9ab5] mb-4 max-w-xl">{desc}</p>

              {/* Terminal block */}
              <div className="rounded border border-[#1e2a36] bg-[#0d1117] overflow-hidden max-w-xl">
                <div className="flex items-center gap-2 px-3 py-2 border-b border-[#1e2a36]">
                  <span className="w-2 h-2 rounded-full bg-[#ff5f57]" />
                  <span className="w-2 h-2 rounded-full bg-[#febc2e]" />
                  <span className="w-2 h-2 rounded-full bg-[#28c840]" />
                </div>
                <div className="px-4 py-3 space-y-1.5">
                  <div className="flex gap-2 text-xs">
                    <span style={{ color }}>$</span>
                    <span className="text-[#7a9ab5]">{cmd}</span>
                  </div>
                  <div
                    className="text-[10px] text-[#3d5a73] leading-relaxed border-l-2 pl-3"
                    style={{ borderColor: `${color}40` }}
                  >
                    {output}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}