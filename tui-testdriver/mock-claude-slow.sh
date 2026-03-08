#!/usr/bin/env bash
# Slow mock claude CLI for testing queue progress.
# Same as mock-claude.sh but with a delay to ensure queue can form.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

# Delay to allow multiple messages to queue up
sleep 0.3

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

printf '\n## Questions\n\n### Q1 (p5): What are the requirements?\n\nPlease elaborate.\n' >> "$SPEC"

echo "I have integrated the requirements."
