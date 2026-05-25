'use client'
import React from "react";

export function Footer() {
  return (
    <footer className="relative z-10 border-t border-[#1e2a36] py-8 px-6">
      <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <span className="text-[#00ff9d]">⚡</span>
          <span
            className="text-xs text-[#3d5a73]"
            style={{ fontFamily: "'Space Mono', monospace" }}
          >
            STELLARFEES
          </span>
        </div>

        <div className="text-[10px] text-[#3d5a73] tracking-wide text-center">
          BUILT WITH RUST · NEXT.JS · STELLAR HORIZON API · OPEN SOURCE
        </div>

        <div className="flex items-center gap-1.5">
          <span className="w-1.5 h-1.5 rounded-full bg-[#00ff9d] animate-pulse" />
          <span className="text-[10px] text-[#3d5a73]">TESTNET LIVE</span>
        </div>
      </div>
    </footer>
  )
}