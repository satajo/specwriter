#!/usr/bin/env bash
# Mock claude CLI for BDD tests.
# Supports stable question pool with IDs.
# Returns different question pools based on prompt content.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

# Simple mock: create/update SPEC.md based on the prompt content
if [ -f "$SPEC" ] && [ -s "$SPEC" ]; then
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
if echo "$PROMPT" | grep -qi "login"; then
    echo "login" >> "$SPEC"
fi
if echo "$PROMPT" | grep -qi "password"; then
    echo "password" >> "$SPEC"
fi
if echo "$PROMPT" | grep -qi "authentication"; then
    echo "authentication" >> "$SPEC"
fi
if echo "$PROMPT" | grep -qi "search"; then
    echo "search" >> "$SPEC"
fi
if echo "$PROMPT" | grep -qi "filter"; then
    echo "filtering" >> "$SPEC"
fi
# If current question pool was passed as context, mark that we're questions-aware
if echo "$PROMPT" | grep -qi "CURRENT QUESTION POOL"; then
    echo "questions-aware" >> "$SPEC"
fi

echo "I have integrated the requirements into SPEC.md."

# Return questions with stable IDs based on prompt content
if echo "$PROMPT" | grep -qi "OAuth"; then
    # Answered Q1 (auth question), keep Q2, Q3, add Q4
    echo 'QUESTIONS:[{"id":2,"text":"Should there be role-based access?"},{"id":3,"text":"What is the target platform?"},{"id":4,"text":"What OAuth providers should be supported?"}]'
elif echo "$PROMPT" | grep -qi "search"; then
    # Keep Q1, Q2, remove Q3, add Q4
    echo 'QUESTIONS:[{"id":1,"text":"What are the authentication requirements?"},{"id":2,"text":"Should there be role-based access?"},{"id":4,"text":"What search fields are needed?"}]'
else
    # Default: return Q1, Q2, Q3
    echo 'QUESTIONS:[{"id":1,"text":"What are the authentication requirements?"},{"id":2,"text":"Should there be role-based access?"},{"id":3,"text":"What is the target platform?"}]'
fi
