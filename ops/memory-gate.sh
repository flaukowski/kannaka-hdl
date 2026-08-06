#!/bin/sh
# Memory integrity gate — pre-flight for a live Kannaka HRM.
#
# Resolves examples/memory-gate.khdl against the running memory in
# strict mode: every anchor must surface above its floor or the gate
# exits nonzero. Run it from cron for drift detection, and ALWAYS
# before one-way operations (.hrm format migrations, encoder swaps,
# facet backfills).
#
#   KANNAKA_HDL_BIN   kannaka-hdl binary   (default: kannaka-hdl on PATH)
#   KANNAKA_BIN       kannaka binary       (default: kannaka on PATH)
#   GATE_FILE         .khdl gate program   (default: alongside this script)
#   GATE_LOG          append log           (default: ~/.kannaka/memory-gate.log)
#
# Exit codes: 0 = all anchors resolved and every expect gate passed;
# nonzero = the memory no longer knows something it must never forget.
# A failure is a finding, not an error: read the warnings, they name
# the anchor that went dark.

set -u

HDL="${KANNAKA_HDL_BIN:-kannaka-hdl}"
KAN="${KANNAKA_BIN:-kannaka}"
HERE="$(cd "$(dirname "$0")" && pwd)"
GATE="${GATE_FILE:-$HERE/../examples/memory-gate.khdl}"
LOG="${GATE_LOG:-$HOME/.kannaka/memory-gate.log}"

STAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OUT="$("$HDL" grow "$GATE" --memory-provider "$KAN" --unresolved strict 2>&1)"
CODE=$?

{
  echo "[$STAMP] exit=$CODE gate=$GATE"
  echo "$OUT" | grep -E "resolved|warning|expect|error" | sed 's/^/  /'
} >> "$LOG"

if [ "$CODE" -ne 0 ]; then
  echo "MEMORY GATE FAILED (exit $CODE) — the HRM no longer resolves:" >&2
  echo "$OUT" | grep -E "warning|error" >&2
fi
exit "$CODE"
