#!/usr/bin/env bash
# Mock that simulates session expiry: fails on --resume, succeeds on --session-id.
# Uses a state file to track whether a fresh session has been started.

WORKDIR="$(pwd)"
STATE_FILE="$WORKDIR/.session-state"

# Check if this is a --resume call (expired session)
for arg in "$@"; do
    if [ "$arg" = "--resume" ]; then
        echo "Session expired or invalid" >&2
        exit 1
    fi
done

# Fresh session (--session-id): succeed
SPEC_DIR="$WORKDIR/specs"
mkdir -p "$SPEC_DIR"
printf '# Spec\n\nRecovered session content.\n' > "$SPEC_DIR/README.md"
echo "Integration complete with fresh session."
