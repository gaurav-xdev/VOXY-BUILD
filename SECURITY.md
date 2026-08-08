# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in VOXY, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please report vulnerabilities via email to: security@voxy-ai.com

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 5 business days
- **Fix timeline**: Depends on severity, typically 1-4 weeks

## Scope

This security policy applies to:
- The VOXY application binary
- The VOXY daemon
- Library crates published to crates.io

## Out of Scope

- Dependencies (report to their maintainers)
- Development/build tools
- Documentation

## Security Best Practices

VOXY follows these security practices:

- No secrets or keys in source code
- No `panic!()` or `unwrap()` on untrusted input
- Proper error handling throughout
- Audit of dependencies via `cargo-audit` and `cargo-deny`
- Minimal attack surface (no unnecessary network APIs)

## Dependency Auditing

We use `cargo-audit` and `cargo-deny` to scan dependencies for known vulnerabilities. These are run as part of our CI pipeline.

## Updates

Security updates are released as patch versions (e.g., 0.1.1 → 0.1.2) and are announced in the CHANGELOG.
