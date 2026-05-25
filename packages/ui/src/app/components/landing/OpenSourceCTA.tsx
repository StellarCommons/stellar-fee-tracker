'use client'
import Link from 'next/link'

const GITHUB = 'https://github.com/StellarCommons/stellar-fee-tracker'

function GitHubIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
    </svg>
  )
}

export function OpenSourceCTA() {
  return (
    <section className="relative z-10 py-24 px-6 border-t border-[#1e2a36]">
      <div className="max-w-2xl mx-auto text-center">
        {/* Badge */}
        <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-[#1e2a36] bg-[#0d1117] mb-8">
          <GitHubIcon />
          <span className="text-xs text-[#7a9ab5]">Open Source · MIT License</span>
        </div>

        {/* Headline */}
        <h2
          className="text-3xl md:text-5xl font-bold text-[#e2eaf4] mb-6"
          style={{ fontFamily: "'Space Mono', monospace", letterSpacing: '-0.02em' }}
        >
          Free to use.<br />
          <span className="text-[#00ff9d]">Free to fork.</span>
        </h2>

        <p className="text-sm text-[#7a9ab5] mb-10 max-w-md mx-auto leading-relaxed">
          Built with Rust, Next.js and Stellar&apos;s Horizon API.
          Star it, fork it, contribute to it.
        </p>

        {/* CTAs */}
        <div className="flex flex-wrap items-center justify-center gap-4">
          <a
            href={GITHUB}
            target="_blank"
            rel="noopener noreferrer"
            className="group flex items-center gap-2 px-6 py-3 bg-[#e2eaf4] text-[#080b0f] rounded font-bold text-sm hover:bg-white transition-all duration-200 hover:-translate-y-0.5"
            style={{ fontFamily: "'Space Mono', monospace" }}
          >
            <GitHubIcon size={16} />
            STAR ON GITHUB
          </a>
          <Link
            href="/dashboard"
            className="flex items-center gap-2 px-6 py-3 border border-[#00ff9d40] text-[#00ff9d] rounded text-sm hover:bg-[#00ff9d08] transition-all duration-200"
            style={{ fontFamily: "'Space Mono', monospace" }}
          >
            OPEN DASHBOARD →
          </Link>
        </div>
      </div>
    </section>
  )
}