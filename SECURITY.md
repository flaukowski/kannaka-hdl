# Security Policy

## Supported Versions

Only the latest tagged release receives security fixes.

## Reporting a Vulnerability

Please report vulnerabilities privately via
[GitHub Security Advisories](https://github.com/flaukowski/kannaka-hdl/security/advisories/new).
Do not open a public issue for exploitable bugs. You should receive a
response within a week.

## Scope Notes

- `.khdl` programs are data, not code: the interpreter has no
  filesystem-write, network, or process primitives. The only file reads
  are the program itself and the registry JSON the user points it at.
- Growth is bounded (depth/leaf limits), so hostile programs cannot hang
  the grower.
- Emitted HTML embeds the plan JSON with `</`-escaping; it contains no
  external resources.
- Dependencies are audited in CI with `cargo audit`
  (`.github/workflows/security.yml`).
