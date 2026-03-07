#!/usr/bin/env bash
# Mock claude CLI for BDD tests.
# Writes to specs/ directory and places questions under ## Questions heading.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/specs"
README="$SPEC_DIR/README.md"

mkdir -p "$SPEC_DIR"

# Build content section (strip existing Questions heading and below)
if [ -f "$README" ] && [ -s "$README" ]; then
    EXISTING=$(sed '/^## Questions$/,$d' "$README" | sed -e :a -e '/^\n*$/{$d;N;ba}')
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

# Place questions under ## Questions heading as ### subheadings
printf '\n## Questions\n\n' >> "$README"
if echo "$PROMPT" | grep -qi "OAuth"; then
    # Answered Q1 (auth question), keep Q2, Q3, add Q4
    printf '### Q2 (p6): Should there be role-based access?\n\nDo different users need different permissions?\n\n### Q3 (p4): What is the target platform?\n\nWeb, mobile, or desktop?\n\n### Q4 (p7): What OAuth providers should be supported?\n\nGoogle, GitHub, etc.?\n' >> "$README"
elif echo "$PROMPT" | grep -qi "search"; then
    # Keep Q1, Q2, remove Q3, add Q4
    printf '### Q1 (p8): What are the authentication requirements?\n\nHow should users authenticate?\n\n### Q2 (p6): Should there be role-based access?\n\nDo different users need different permissions?\n\n### Q4 (p5): What search fields are needed?\n\nWhich fields should be searchable?\n' >> "$README"
else
    # Default: Q1, Q2, Q3
    printf '### Q1 (p8): What are the authentication requirements?\n\nHow should users authenticate?\n\n### Q2 (p6): Should there be role-based access?\n\nDo different users need different permissions?\n\n### Q3 (p4): What is the target platform?\n\nWeb, mobile, or desktop?\n' >> "$README"
fi

echo "I have integrated the requirements."
