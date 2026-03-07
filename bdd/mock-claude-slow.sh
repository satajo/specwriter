#!/usr/bin/env bash
# Slow mock claude CLI for testing queue progress.
# Same as mock-claude.sh but with a delay to ensure queue can form.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/spec"
README="$SPEC_DIR/README.md"

# Delay to allow multiple messages to queue up
sleep 0.3

mkdir -p "$SPEC_DIR"

# Create or update base content
if [ -f "$README" ] && [ -s "$README" ]; then
    EXISTING=$(grep -v "^?Q" "$README" || true)
    printf '%s\n\n---\n\nUpdated with new requirements.\n' "$EXISTING" > "$README"
else
    printf '# Spec\n\nRequirements integrated from user input.\n' > "$README"
fi

# Extract keywords
echo "$PROMPT" | grep -qi "search" && echo "search" >> "$README"
echo "$PROMPT" | grep -qi "filter" && echo "filtering" >> "$README"

echo "" >> "$README"
echo "?Q1: What are the requirements?" >> "$README"

echo "I have integrated the requirements."
