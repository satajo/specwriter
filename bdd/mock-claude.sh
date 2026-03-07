#!/usr/bin/env bash
# Mock claude CLI for BDD tests.
# Writes to spec/ directory and embeds inline questions.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/spec"
README="$SPEC_DIR/README.md"

mkdir -p "$SPEC_DIR"

# Create or update base content (strip existing question lines)
if [ -f "$README" ] && [ -s "$README" ]; then
    EXISTING=$(grep -v "^?Q" "$README" || true)
    printf '%s\n\n---\n\nUpdated with new requirements from the latest integration.\n' "$EXISTING" > "$README"
else
    printf '# Spec\n\nRequirements integrated from user input.\n' > "$README"
fi

# Extract keywords from the prompt to make spec reflective
echo "$PROMPT" | grep -qi "login" && echo "login" >> "$README"
echo "$PROMPT" | grep -qi "password" && echo "password" >> "$README"
echo "$PROMPT" | grep -qi "authentication" && echo "authentication" >> "$README"
echo "$PROMPT" | grep -qi "search" && echo "search" >> "$README"
echo "$PROMPT" | grep -qi "filter" && echo "filtering" >> "$README"

# Embed inline questions based on prompt content
echo "" >> "$README"
if echo "$PROMPT" | grep -qi "OAuth"; then
    # Answered Q1 (auth question), keep Q2, Q3, add Q4
    echo "?Q2: Should there be role-based access?" >> "$README"
    echo "?Q3: What is the target platform?" >> "$README"
    echo "?Q4: What OAuth providers should be supported?" >> "$README"
elif echo "$PROMPT" | grep -qi "search"; then
    # Keep Q1, Q2, remove Q3, add Q4
    echo "?Q1: What are the authentication requirements?" >> "$README"
    echo "?Q2: Should there be role-based access?" >> "$README"
    echo "?Q4: What search fields are needed?" >> "$README"
else
    # Default: Q1, Q2, Q3
    echo "?Q1: What are the authentication requirements?" >> "$README"
    echo "?Q2: Should there be role-based access?" >> "$README"
    echo "?Q3: What is the target platform?" >> "$README"
fi

echo "I have integrated the requirements."
