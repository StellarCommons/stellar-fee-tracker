# Security Policy

## Supported Versions

Stellar Fee Tracker is maintained as a TypeScript monorepo (`@stellar-fee-tracker/core`, `@stellar-fee-tracker/ui`, and `@stellar-fee-tracker/devkit`). Security fixes are applied to the latest release line on the `main` branch.

| Version | Supported |
| ------- | --------- |
| `0.1.x` (`main`) | :white_check_mark: |
| `< 0.1.0` | :x: |

## Security Scope

We particularly welcome reports covering:

- **API Authentication & Rate Limiting:** Timing-safe API key verification (`X-API-Key`), rate-limiter bypasses, or unauthorized access to protected alert and webhook endpoints.
- **Webhook & SSRF Protection:** Outbound webhook URL validation, DNS rebinding / private IP range bypasses, or unsafe payload serialization in `AlertEngine`.
- **Collector & Storage Integrity:** SQL injection, unbounded memory growth, or denial-of-service vectors in Horizon fee ingestion and SQLite storage queries.
- **Dashboard & Frontend Security:** Cross-site scripting (XSS), unsafe clipboard or storage handling, and dependency vulnerabilities.

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues, discussions, or pull requests.**

Instead, please report suspected vulnerabilities privately using one of the following channels:

1. **GitHub Private Vulnerability Reporting (Preferred):**
   Open a private advisory directly at [`https://github.com/StellarCommons/stellar-fee-tracker/security/advisories/new`](https://github.com/StellarCommons/stellar-fee-tracker/security/advisories/new).
2. **Maintainer Contact:**
   If GitHub Security Advisories are unavailable, contact the StellarCommons maintainers via the organization profile on GitHub with the subject line `[SECURITY] stellar-fee-tracker`.

### What to Include

To help us triage and resolve the issue quickly, please include:

- The affected package(s) (`packages/core`, `packages/ui`, or `packages/devkit`) and file path(s).
- A clear description of the vulnerability and its potential impact.
- Step-by-step reproduction instructions or a minimal proof-of-concept.
- Any suggested mitigations or patches, if available.

## Response Timeline

- **Acknowledgement:** Within **48 hours** of receiving a report.
- **Triage & Severity Assessment:** Within **5 business days**.
- **Remediation & Coordinated Disclosure:** Critical and high-severity fixes are prioritized for an immediate patch release. We ask that reporters allow up to **90 days** (or until a patch is published) before public disclosure.
