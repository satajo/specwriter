#!/usr/bin/env bash
# Mock claude CLI for BDD tests.
# Reads the prompt (last arg), writes a SPEC.md, and outputs questions.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

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
if echo "$PROMPT" | grep -qi "project"; then
    echo "projects" >> "$SPEC"
fi

# If open questions were passed as context, mark that we're questions-aware
if echo "$PROMPT" | grep -qi "OPEN QUESTIONS"; then
    echo "questions-aware" >> "$SPEC"
fi

echo "I have integrated the requirements into SPEC.md."

# Output different questions based on prompt content
if echo "$PROMPT" | grep -qi "search\|filter"; then
    echo 'QUESTIONS:["What search fields are needed?","Should filtering support multiple criteria?"]'
elif echo "$PROMPT" | grep -qi "project"; then
    echo 'QUESTIONS:["How are projects organized?","What project metadata is needed?"]'
else
    echo 'QUESTIONS:["What are the authentication requirements?","Should there be role-based access?","What is the target platform?"]'
fi
