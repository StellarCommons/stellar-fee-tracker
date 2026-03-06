"use client";

import Nav from "./components/landing/Nav";
import Hero from "./components/landing/Hero";
import StatStrip from "./components/landing/StatStrip";
import MockDashboard from "./components/landing/MockDashboard";
import { FeatureOrbit } from "./components/landing/FeatureOrbit";
import { HowItWorks } from "./components/landing/HowItWorks";
import { OpenSourceCTA } from "./components/landing/OpenSourceCTA";
import { Footer } from "./components/landing/Footer";
import { GridBeams } from "./components/landing/GridBeams";

export default function LandingPage() {
  return (
    <div
      style={{
        fontFamily: "'JetBrains Mono', monospace",
        background: "#080b0f",
        color: "#e2eaf4",
      }}
      className="min-h-screen overflow-x-hidden"
    >
      {/* Scanline texture */}
      <div
        className="pointer-events-none fixed inset-0 z-50"
        style={{
          background:
            "repeating-linear-gradient(0deg,transparent,transparent 2px,rgba(0,0,0,0.025) 2px,rgba(0,0,0,0.025) 4px)",
        }}
      />

      {/* Grid bg */}
      <div
        className="pointer-events-none fixed inset-0"
        style={{
          backgroundImage:
            "linear-gradient(rgba(0,255,157,0.025) 1px,transparent 1px),linear-gradient(90deg,rgba(0,255,157,0.025) 1px,transparent 1px)",
          backgroundSize: "48px 48px",
        }}
      />
      <GridBeams />

      <Nav />
      <Hero />
      <StatStrip />

      {/* Dashboard Preview */}
      <section className="relative z-10 py-24 px-6">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16">
            <div className="text-[10px] text-[#00ff9d] tracking-widest mb-3">
              DASHBOARD PREVIEW
            </div>
            <h2
              style={{ fontFamily: "'Space Mono', monospace" }}
              className="text-3xl md:text-4xl font-bold text-[#e2eaf4]"
            >
              Everything in one view.
            </h2>
            <p className="text-sm text-[#7a9ab5] mt-4 max-w-md mx-auto">
              From base fee to p99 distribution — the full picture of
              what&apos;s happening on the Stellar network right now.
            </p>
          </div>
          <div className="relative">
            <div className="absolute inset-0 bg-[#00ff9d] opacity-[0.03] rounded-full blur-3xl scale-75" />
            <MockDashboard />
          </div>
        </div>
      </section>

      <FeatureOrbit />
      <HowItWorks />
      <OpenSourceCTA />
      <Footer />
    </div>
  );
}
