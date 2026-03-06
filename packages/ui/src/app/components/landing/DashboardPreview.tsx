'use client'
import React from "react";
import MockDashboard from "./MockDashboard"

const ANNOTATIONS = [
  { num: '01', label: 'Stat Cards',          desc: 'Base fee, avg, status and spikes at a glance' },
  { num: '02', label: 'Fee History Chart',   desc: 'Bar chart across 1H, 6H or 24H windows with spike detection' },
  { num: '03', label: 'Trend + Averages',    desc: 'Rolling short/medium/long term analysis with % change' },
]

export function DashboardPreview() {
  return (
    <section className="relative z-10 py-24 px-6">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-16">
          <div className="text-[10px] text-[#00ff9d] tracking-widest mb-3">DASHBOARD PREVIEW</div>
          <h2
            className="text-3xl md:text-4xl font-bold text-[#e2eaf4]"
            style={{ fontFamily: "'Space Mono', monospace" }}
          >
            Everything in one view.
          </h2>
          <p className="text-sm text-[#7a9ab5] mt-4 max-w-md mx-auto">
            From base fee to p99 distribution — the full picture of what&apos;s
            happening on the Stellar network right now.
          </p>
        </div>

        {/* Mock with glow */}
        <div className="relative">
          <div className="absolute inset-0 bg-[#00ff9d] opacity-[0.03] rounded-full blur-3xl scale-75" />
          <MockDashboard />
        </div>

        {/* Annotations */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mt-12">
          {ANNOTATIONS.map(({ num, label, desc }) => (
            <div key={num} className="flex gap-4 items-start">
              <div
                className="text-[10px] text-[#00ff9d40] font-bold mt-0.5"
                style={{ fontFamily: "'Space Mono', monospace" }}
              >
                {num}
              </div>
              <div>
                <div
                  className="text-sm text-[#e2eaf4] font-bold mb-1"
                  style={{ fontFamily: "'Space Mono', monospace" }}
                >
                  {label}
                </div>
                <div className="text-xs text-[#3d5a73]">{desc}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}