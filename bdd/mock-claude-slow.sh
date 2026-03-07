#!/usr/bin/env bash
# Slow mock claude CLI for testing queue progress.
# Same as mock-claude.sh but with a delay to ensure queue can form.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

# Delay to allow multiple messages to queue up
sleep 0.3

# Simple mock: create/update SPEC.md based on the prompt content
if [ -f "$SPEC" ]; then
    EXISTING=$(cat "$SPEC")
    cat > "$SPEC" << EOF
$EXISTING

---

Updated with new requirements from the latest integration.
EOF
else
    cat > "$SPEC" << EOF
# Spec

Requirements integrated from user input.
EOF
fi

# Extract keywords from the prompt to make SPEC.md somewhat reflective
if echo "$PROMPT" | grep -qi "search"; then
    echo "search" >> "$SPEC"
fi
if echo "$PROMPT" | grep -qi "filter"; then
    echo "filtering" >> "$SPEC"
fi

echo "I have integrated the requirements into SPEC.md."
echo 'QUESTIONS:[{"id":1,"text":"What are the requirements?"}]'
