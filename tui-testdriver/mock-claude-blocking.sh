#!/usr/bin/env bash
# Blocking mock claude CLI for testing.
# Waits for a signal file before completing, giving tests deterministic control.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

# Wait for signal file to proceed
SIGNAL="$WORKDIR/.mock-proceed"
WAITED=0
while [ ! -f "$SIGNAL" ]; do
    sleep 0.01
    WAITED=$((WAITED + 1))
    [ "$WAITED" -ge 1000 ] && exit 1  # 10s safety timeout
done
rm -f "$SIGNAL"

# Create or update base content
if [ -f "$SPEC" ] && [ -s "$SPEC" ]; then
    EXISTING=$(sed '/^## Questions$/,$d' "$SPEC" | sed -e :a -e '/^\n*$/{$d;N;ba}')
    printf '%s\n\n---\n\nUpdated with new requirements.\n' "$EXISTING" > "$SPEC"
else
    printf '# Spec\n\nRequirements integrated from user input.\n' > "$SPEC"
fi

# Extract keywords
echo "$PROMPT" | grep -qi "search" && echo "search" >> "$SPEC"
echo "$PROMPT" | grep -qi "filter" && echo "filtering" >> "$SPEC"

printf '\n## Questions\n\n### Q1 (p3): What are the requirements?\n\nPlease elaborate.\n' >> "$SPEC"

echo "I have integrated the requirements."
